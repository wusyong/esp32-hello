use std::thread;
use std::net::{UdpSocket, SocketAddr, SocketAddrV4};
use std::mem::{size_of, size_of_val, transmute};
use std::mem::{self, MaybeUninit};
use std::fmt;
use std::str;
use std::net::{Ipv4Addr, ToSocketAddrs};
use std::iter::Iterator;

const MAX_MESSAGE_SIZE: usize = 512;
const HEADER_SIZE: usize = size_of::<DnsHeader>();

pub fn respond(header: &mut DnsHeader, response_code: ResponseCode) {
  header.set_type(HeaderType::Response);
  header.set_response_code(response_code);
  header.set_question_count(0);
  header.set_answer_count(0);
  header.set_name_server_count(0);
  header.set_additional_records_count(0);
}

pub fn server() {
  println!("Starting DNS server …");

  let mut socket = UdpSocket::bind("0.0.0.0:53").unwrap();

  'outer: loop {
    let mut buf = [0; MAX_MESSAGE_SIZE];
    let (len, src) = socket.recv_from(&mut buf).expect("recv_from failed");

    println!("Request from {:?} with len {}: {:?}", src, len, &buf[..len]);

    thread::yield_now();

    if len < HEADER_SIZE || len > MAX_MESSAGE_SIZE {
      continue
    }

    let mut buf = &mut buf[..len];

    let (mut header, mut rest) = buf.split_at_mut(HEADER_SIZE);

    let mut header = unsafe { &mut *(header as *mut _ as *mut DnsHeader) };

    println!("DNS Header: {:?}", header);

    if header.header_type() != HeaderType::Query {
      continue
    }

    // HACK: For some reason, `src` becomes a null-pointer somewhere in
    // `impl ToSocketAddrs for SocketAddr`.
    let src = &[src][..];

    if header.opcode() == OpCode::Query {
      let rest = &*rest;

      let mut question_iter = DnsIter {
        current_question: 0,
        question_count: header.question_count() as usize,
        buf: rest,
        buf_i: 0,
      };

      while let Some(res) = question_iter.next() {
        match res {
          Ok((s, t, c)) => {
            println!("DOMAIN: {:?}", s);

            if s == "captive.apple.com" {
              header.set_type(HeaderType::Response);
              header.set_response_code(ResponseCode::NoError);
              header.set_answer_count(header.question_count());

              let mut response = header.as_bytes().to_vec();

              response.extend(rest);

              response.extend(&(0xC000 | size_of::<DnsHeader>() as u16).to_be_bytes());

              response.extend(&t.to_be_bytes());
              response.extend(&c.to_be_bytes());

              let ttl: u32 = 60;
              response.extend(&ttl.to_be_bytes());

              let gateway_ip = Ipv4Addr::new(192, 168, 4, 1);
              response.extend(&(size_of_val(&gateway_ip) as u16).to_be_bytes());
              response.extend(&gateway_ip.octets());

              println!("send_to: {:?}", socket.send_to(&response, src));

              continue 'outer;
            } else {
              respond(header, ResponseCode::NameError);
              break;
            }
          },
          Err(response_code) => {
            respond(header, response_code);
            break;
          },
        }
      }
    } else {
      respond(header, ResponseCode::NotImplemented);
    }

    println!("send_to: {:?}", socket.send_to(header.as_bytes(), src));
  }
}

pub struct DnsIter<'a> {
  question_count: usize,
  current_question: usize,
  buf: &'a [u8],
  buf_i: usize,
}

impl<'a> Iterator for DnsIter<'a> {
  type Item = Result<(String, QueryType, QueryClass), ResponseCode>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current_question >= self.question_count {
      return None
    }

    let mut i = self.buf_i;
    let mut domain = String::new();

    loop {
      let part_len = if let Some(&len) = self.buf.get(i) {
        len as usize
      } else {
        return Some(Err(ResponseCode::FormatError))
      };

      if part_len == 0 {
        let query_type = if let (Some(&b1), Some(&b2)) = (self.buf.get(i + 1), self.buf.get(i + 2)) {
          QueryType::from(u16::from_be_bytes([b1, b2]))
        } else {
          return Some(Err(ResponseCode::FormatError))
        };

        let query_class = if let (Some(&b1), Some(&b2)) = (self.buf.get(i + 3), self.buf.get(i + 4)) {
          QueryClass::from(u16::from_be_bytes([b1, b2]))
        } else {
          return Some(Err(ResponseCode::FormatError))
        };

        self.current_question += 1;
        self.buf_i += i;

        return Some(Ok((domain, query_type, query_class)))
      } else {
        if !domain.is_empty() {
          domain.push('.');
        }

        if let Some(part) = self.buf.get((i + 1)..(i + part_len + 1)) {
          domain.push_str(unsafe { str::from_utf8_unchecked(part) });
        } else {
          return Some(Err(ResponseCode::FormatError))
        }
      }

      i += part_len + 1;
    }
  }
}

pub struct DnsHeader {
  id: [u8; 2],
  flags: [u8; 2],
  question_count: [u8; 2],
  answer_count: [u8; 2],
  name_server_count: [u8; 2],
  additional_records_count: [u8; 2],
}

#[derive(Debug, PartialEq)]
pub enum HeaderType {
  Query,
  Response,
}

#[derive(Debug, PartialEq)]
pub enum OpCode {
  Query,
  InverseQuery,
  Status,
  Notify,
  Update,
  Reserved(u8)
}

#[derive(Debug, PartialEq)]
pub enum ResponseCode {
  NoError,
  FormatError,
  ServerFailure,
  NameError,
  NotImplemented,
  Refused,
  Reserved(u8),
}

/// https://tools.ietf.org/rfc/rfc1035.txt
#[derive(Debug, PartialEq)]
pub enum QueryType {
  A,
  NS,
  MD,
  MF,
  CNAME,
  SOA,
  MB,
  MG,
  MR,
  NULL,
  WKS,
  PTR,
  HINFO,
  MINFO,
  MX,
  TXT,
  AXFR,
  MAILA,
  MAILB,
  ALL,
}

impl From<u16> for QueryType {
  fn from(n: u16) -> Self {
    match n {
      1 => Self::A,
      2 => Self::NS,
      3 => Self::MD,
      4 => Self::MF,
      5 => Self::CNAME,
      6 => Self::SOA,
      7 => Self::MB,
      8 => Self::MG,
      9 => Self::MR,
      10 => Self::NULL,
      11 => Self::WKS,
      12 => Self::PTR,
      13 => Self::HINFO,
      14 => Self::MINFO,
      15 => Self::MX,
      16 => Self::TXT,
      252 => Self::AXFR,
      253 => Self::MAILB,
      254 => Self::MAILA,
      255 => Self::ALL,
      _ => unreachable!(),
    }
  }
}

impl QueryType {
  fn to_be_bytes(&self) -> [u8; 2] {
    match self {
      Self::A => u16::to_be_bytes(1),
      Self::NS => u16::to_be_bytes(2),
      Self::MD => u16::to_be_bytes(3),
      Self::MF => u16::to_be_bytes(4),
      Self::CNAME => u16::to_be_bytes(5),
      Self::SOA => u16::to_be_bytes(6),
      Self::MB => u16::to_be_bytes(7),
      Self::MG => u16::to_be_bytes(8),
      Self::MR => u16::to_be_bytes(9),
      Self::NULL => u16::to_be_bytes(10),
      Self::WKS => u16::to_be_bytes(11),
      Self::PTR => u16::to_be_bytes(12),
      Self::HINFO => u16::to_be_bytes(13),
      Self::MINFO => u16::to_be_bytes(14),
      Self::MX => u16::to_be_bytes(15),
      Self::TXT => u16::to_be_bytes(16),
      Self::AXFR => u16::to_be_bytes(252),
      Self::MAILB => u16::to_be_bytes(253),
      Self::MAILA => u16::to_be_bytes(254),
      Self::ALL => u16::to_be_bytes(255),
      _ => unreachable!(),
    }
  }
}


/// https://tools.ietf.org/rfc/rfc1035.txt
#[derive(Debug, PartialEq)]
pub enum QueryClass {
  IN,
  CS,
  CH,
  HS,
}

impl QueryClass {
  fn to_be_bytes(&self) -> [u8; 2] {
    match self {
      Self::IN => u16::to_be_bytes(1),
      Self::CS => u16::to_be_bytes(2),
      Self::CH => u16::to_be_bytes(3),
      Self::HS => u16::to_be_bytes(4),
      _ => unreachable!(),
    }
  }
}

impl From<u16> for QueryClass {
  fn from(n: u16) -> Self {
    match n {
      1 => Self::IN,
      2 => Self::CS,
      3 => Self::CH,
      4 => Self::HS,
      _ => unreachable!(),
    }
  }
}

impl fmt::Debug for DnsHeader {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    fmt.debug_struct("DnsHeader")
      .field("id", &self.id())
      .field("header_type", &self.header_type())
      .field("opcode", &self.opcode())
      .field("authoritative_answer", &self.authoritative_answer())
      .field("truncated", &self.truncated())
      .field("recursion_desired", &self.recursion_desired())
      .field("recursion_available", &self.recursion_available())
      .field("response_code", &self.response_code())
      .field("question_count", &self.question_count())
      .field("answer_count", &self.answer_count())
      .field("name_server_count", &self.name_server_count())
      .field("additional_records_count", &self.additional_records_count())
      .finish()
  }
}

impl DnsHeader {
  pub fn id(&self) -> u16 {
    u16::from_be_bytes(self.id)
  }

  pub fn header_type(&self) -> HeaderType {
    if (self.flags[0] & 0b10000000) == 0 {
      HeaderType::Query
    } else {
      HeaderType::Response
    }
  }

  pub fn set_type(&mut self, header_type: HeaderType) {
    match header_type {
      HeaderType::Query    => self.flags[0] &= 0b01111111,
      HeaderType::Response => self.flags[0] |= 0b10000000,
    }
  }

  pub fn opcode(&self) -> OpCode {
    match (self.flags[0] & 0b01111000) >> 3 {
      0 => OpCode::Query,
      1 => OpCode::InverseQuery,
      2 => OpCode::Status,
      4 => OpCode::Notify,
      5 => OpCode::Update,
      n => OpCode::Reserved(n),
    }
  }

  pub fn set_opcode(&mut self, opcode: OpCode) {
    self.flags[0] = (self.flags[0] & 0b10000111) | (match opcode {
      OpCode::Query => 0,
      OpCode::InverseQuery => 1,
      OpCode::Status => 2,
      OpCode::Notify => 4,
      OpCode::Update => 5,
      OpCode::Reserved(n) => n,
    } << 3);
  }

  pub fn authoritative_answer(&self) -> bool {
    (self.flags[0] & 0b00000100) != 0
  }

  pub fn truncated(&self) -> bool {
    (self.flags[0] & 0b00000010) != 0
  }

  pub fn recursion_desired(&self) -> bool {
    (self.flags[0] & 0b00000001) != 0
  }

  pub fn recursion_available(&self) -> bool {
    (self.flags[1] & 0b10000000) != 0
  }

  pub fn response_code(&self) -> ResponseCode {
    match self.flags[1] & 0b00001111 {
      0 => ResponseCode::NoError,
      1 => ResponseCode::FormatError,
      2 => ResponseCode::ServerFailure,
      3 => ResponseCode::NameError,
      4 => ResponseCode::NotImplemented,
      5 => ResponseCode::Refused,
      n => ResponseCode::Reserved(n),
    }
  }

  pub fn set_response_code(&mut self, response_code: ResponseCode) {
    self.flags[1] = (self.flags[1] & 0b11110000) | match response_code {
      ResponseCode::NoError => 0,
      ResponseCode::FormatError => 1,
      ResponseCode::ServerFailure => 2,
      ResponseCode::NameError => 3,
      ResponseCode::NotImplemented => 4,
      ResponseCode::Refused => 5,
      ResponseCode::Reserved(n) => n,
    };
  }

  pub fn question_count(&self) -> u16 {
    u16::from_be_bytes(self.question_count)
  }

  pub fn set_question_count(&mut self, question_count: u16) {
    self.question_count = question_count.to_be_bytes();
  }

  pub fn answer_count(&self) -> u16 {
    u16::from_be_bytes(self.answer_count)
  }

  pub fn set_answer_count(&mut self, answer_count: u16) {
    self.answer_count = answer_count.to_be_bytes();
  }

  pub fn name_server_count(&self) -> u16 {
    u16::from_be_bytes(self.name_server_count)
  }

  pub fn set_name_server_count(&mut self, name_server_count: u16) {
    self.name_server_count = name_server_count.to_be_bytes();
  }

  pub fn additional_records_count(&self) -> u16 {
    u16::from_be_bytes(self.additional_records_count)
  }

  pub fn set_additional_records_count(&mut self, additional_records_count: u16) {
    self.additional_records_count = additional_records_count.to_be_bytes();
  }

  pub fn as_bytes(&self) -> &[u8] {
    unsafe { &*(self as *const _ as *const [u8; size_of::<Self>()]) }
  }
}
