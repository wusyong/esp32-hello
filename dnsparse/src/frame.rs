use core::ops::{Deref, Range};
use core::mem::{size_of, transmute};
use core::fmt;

use crate::{DnsHeader, ResponseCode, QueryKind, QueryClass, Question, Questions};

const HEADER_SIZE: usize = size_of::<DnsHeader>();
const MAX_MESSAGE_SIZE: usize = 512;

#[repr(C)]
pub struct DnsFrame {
  header: DnsHeader,
  msg: [u8; MAX_MESSAGE_SIZE],
  len: usize,
}

impl fmt::Debug for DnsFrame {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    fmt.debug_struct("DnsFrame")
      .field("header", &self.header)
      .field("msg", &format_args!("{:?}", &self.msg[..]))
      .field("len", &self.len)
      .finish()
  }
}

impl Deref for DnsFrame {
  type Target = [u8];

  fn deref(&self) -> &Self::Target {
    self.as_bytes()
  }
}

const DOT: u8 = '.' as u32 as u8;

impl DnsFrame {
  pub fn new(header: DnsHeader) -> Self {
    Self {
      header,
      msg: [0; MAX_MESSAGE_SIZE],
      len: 0,
    }
  }

  pub fn header(&self) -> &DnsHeader {
    &self.header
  }

  pub fn header_mut(&mut self) -> &mut DnsHeader {
    &mut self.header
  }

  pub fn body(&self) -> &[u8] {
    &self.msg[..self.len]
  }

  pub fn add_question(&mut self, question: &Question) {
    let mut i = 0;

    let question = question.as_bytes();

    while question[i] == DOT {
      let part_len = question[(i + 1)..].iter().take_while(|&&b| b != DOT && b != 0).count();
      self.extend(&[part_len as u8]);
      self.extend(&question[(i + 1)..(i + 1 + part_len)]);
      i += part_len + 1;
    }

    self.extend(&question[i..(i + 5)]);
  }

  pub fn add_ttl(&mut self, ttl: u32) {
    self.extend(&ttl.to_be_bytes());
  }

  pub fn add_rdata(&mut self, data: &[u8]) {
    self.extend(&(data.len() as u16).to_be_bytes());
    self.extend(data);
  }

  pub fn extend(&mut self, bytes: &[u8]) {
    self.msg[self.len..(self.len + bytes.len())].copy_from_slice(bytes);
    self.len += bytes.len();
  }

  pub unsafe fn set_len(&mut self, mut len: usize) {
    debug_assert!(len >= HEADER_SIZE && len <= (HEADER_SIZE + MAX_MESSAGE_SIZE));
    self.len = len - HEADER_SIZE;
  }

  pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
    &mut *(&mut self.header as *mut _ as *mut [u8; HEADER_SIZE + MAX_MESSAGE_SIZE])
  }

  pub fn as_bytes(&self) -> &[u8] {
    unsafe {
      let len = HEADER_SIZE + self.len;
      &(*(&self.header as *const _ as *const [u8; HEADER_SIZE + MAX_MESSAGE_SIZE] ))[..len]
    }
  }

  pub fn questions(&self) -> Questions {
    Questions {
      question_count: self.header.question_count() as usize,
      current_question: 0,
      buf: &self.body(),
      buf_i: 0,
    }
  }
}

impl From<DnsHeader> for DnsFrame {
  fn from(header: DnsHeader) -> Self {
    Self::new(header)
  }
}
