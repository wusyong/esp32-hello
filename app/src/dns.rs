use std::thread;
use std::net::{UdpSocket, SocketAddr, SocketAddrV4};
use std::mem::{size_of, size_of_val, transmute};
use std::mem::{self, MaybeUninit};
use std::fmt;
use std::str;
use std::net::{Ipv4Addr, ToSocketAddrs};
use std::iter::Iterator;
use std::ops::Range;
use std::ops::Deref;

use esp_idf_sys::{tcpip_adapter_get_ip_info, tcpip_adapter_if_t, tcpip_adapter_ip_info_t};

use dnsparse::*;

pub fn handle_request(request: DnsFrame, ip: &Ipv4Addr) -> DnsFrame {
  let mut header = request.header().clone();
  header.set_kind(HeaderKind::Response);
  header.set_recursion_available(header.recursion_desired());
  header.set_question_count(0);
  header.set_answer_count(0);
  header.set_name_server_count(0);
  header.set_additional_records_count(0);
  header.set_response_code(ResponseCode::NotImplemented);

  let mut response = DnsFrame::new(header);

  if request.header().question_count() != 1 {
    return response
  }

  if request.header().kind() != HeaderKind::Query {
    return response
  }

  if request.header().opcode() != OpCode::Query {
    return response
  }

  let mut questions = request.questions();

  match questions.next().unwrap() {
    Ok(question) => if question.kind() == QueryKind::A && question.class() == QueryClass::IN {
      if question.name() == "captive.apple.com" {
        {
          let header = response.header_mut();
          header.set_response_code(ResponseCode::NoError);
          header.set_answer_count(header.answer_count() + 1);
        }

        response.add_question(&question);
        response.add_ttl(60);
        response.add_rdata(&ip.octets());
      } else {
        response.header_mut().set_response_code(ResponseCode::NonExistentDomain);
      }
    },
    Err(response_code) => {
      response.header_mut().set_response_code(response_code);
    },
  }

  response
}

#[derive(Debug)]
struct IpInfo {
  ip: Ipv4Addr,
  netmask: Ipv4Addr,
  gw: Ipv4Addr,
}

pub fn server() {
  println!("Starting DNS server …");

  let mut info = MaybeUninit::<tcpip_adapter_ip_info_t>::uninit();
  let info: IpInfo = unsafe {
    tcpip_adapter_get_ip_info(tcpip_adapter_if_t::TCPIP_ADAPTER_IF_AP, info.as_mut_ptr());
    transmute(info.assume_init())
  };

  let mut socket = UdpSocket::bind("0.0.0.0:53").unwrap();

  'outer: loop {
    thread::yield_now();

    let (mut request, src) = unsafe {
      let mut frame = MaybeUninit::<DnsFrame>::uninit();

      let (len, src) = match socket.recv_from((&mut *frame.as_mut_ptr()).as_mut_slice()) {
        Ok(ok) => ok,
        Err(err) => {
          eprintln!("DNS received failed: {:?}", err);
          continue
        }
      };

      if len < size_of::<DnsHeader>() {
        continue
      }

      (&mut *frame.as_mut_ptr()).set_len(len);

      (frame.assume_init(), src)
    };

    let response = handle_request(request, &info.ip);

    if let Err(err) = socket.send_to(&response, src) {
      eprintln!("Error sending response to '{:?}': {}", src, err);
    }
  }
}
