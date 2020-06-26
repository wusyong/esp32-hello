use std::net::{Ipv4Addr, UdpSocket};
use std::thread;

use esp_idf_hal::interface::Interface;

use dnsparse::*;

pub fn handle_request(request: DnsFrame, ip: &Ipv4Addr) -> DnsFrame {
  let response_header = DnsHeader::builder()
    .id(request.header().id())
    .kind(HeaderKind::Response)
    .recursion_available(request.header().recursion_desired())
    .response_code(ResponseCode::NotImplemented);

  let mut response = DnsFrame::new(response_header.build());

  let question_count = request.header().question_count();
  let kind = request.header().kind();
  let opcode = request.header().opcode();

  if question_count != 1 || kind != HeaderKind::Query || opcode != OpCode::Query {
    return response
  }

  for question in request.questions() {
    match question {
      Ok(question) => {
        if question.kind() == QueryKind::A && question.class() == QueryClass::IN {
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
            break;
          }
        }
      },
      Err(response_code) => {
        response.header_mut().set_response_code(response_code);
      },
    }
  }

  response
}


pub fn server() {
  println!("Starting DNS server …");

  let socket = UdpSocket::bind("0.0.0.0:53").unwrap();

  let ip = *Interface::Ap.ip_info().ip();
  println!("IP: {:?}", ip);

  'outer: loop {
    thread::yield_now();

    let mut buf = DnsFrame::BUFFER;

    let (len, src) = match socket.recv_from(&mut buf) {
      Ok(ok) => ok,
      Err(err) => {
        eprintln!("Receiving DNS request failed: {}", err);
        continue
      }
    };

    let request = if let Ok(frame) = DnsFrame::parse(buf, len) {
      frame
    } else {
      continue 'outer
    };

    let response = handle_request(request, &ip);

    if let Err(err) = socket.send_to(&response, src) {
      eprintln!("Error sending response to '{:?}': {}", src, err);
    }
  }
}
