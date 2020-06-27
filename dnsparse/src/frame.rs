use core::ops::Deref;
use core::mem::{replace, size_of, transmute};
use core::fmt;

use crate::{DnsHeader, Question, Questions};
use crate::question::read_question;

const HEADER_SIZE: usize = size_of::<DnsHeader>();
const MAX_MESSAGE_SIZE: usize = 512 - HEADER_SIZE;

pub type DnsFrameBuffer = [u8; HEADER_SIZE + MAX_MESSAGE_SIZE];

/// A DNS frame.
#[repr(C)]
pub struct DnsFrame<'a> {
  buf: &'a mut [u8],
  len: usize,
}

pub struct DnsFrameBuilder<'a> {
  buf: &'a mut [u8],
  len: usize,
}

impl<'a> DnsFrameBuilder<'a> {
  pub fn header(mut self, header: DnsHeader) -> Self {
    unsafe {
      replace(self.header_mut(), header);
    }

    self.len = HEADER_SIZE;

    self
  }

  fn header_mut(&mut self) -> &mut DnsHeader {
    unsafe { &mut *(self.buf[..HEADER_SIZE].as_mut_ptr() as *mut _ as *mut DnsHeader) }
  }

  pub fn build(self) -> DnsFrame<'a> {
    let Self { buf, len } = self;
    DnsFrame { buf, len }
  }
}

impl<'a> DnsFrame<'a> {
  pub const BUFFER: [u8; 512] = [0; 512];

  pub fn builder(buffer: &'a mut [u8]) -> DnsFrameBuilder<'a> {
    let mut builder = DnsFrameBuilder { buf: buffer, len: HEADER_SIZE };
    builder
  }

  pub fn parse(buffer: &'a mut [u8]) -> Result<DnsFrame<'a>, ()> {
    if buffer.len() < HEADER_SIZE || buffer.len() > HEADER_SIZE + MAX_MESSAGE_SIZE {
      return Err(())
    }

    let len = buffer.len();
    let mut frame = Self { buf: buffer, len };

    let mut i = HEADER_SIZE;

    for _ in 0..frame.header().question_count() {
      if !read_question(&frame.buf, &mut i) {
        return Err(())
      }
    }

    frame.len = i;

    Ok(frame)
  }
}

impl fmt::Debug for DnsFrame<'_> {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    fmt.debug_struct("DnsFrame")
      .field("header", &self.header())
      .field("body", &format_args!("{:?}", &self.buf[HEADER_SIZE..]))
      .field("len", &self.len)
      .finish()
  }
}

impl Deref for DnsFrame<'_> {
  type Target = [u8];

  fn deref(&self) -> &Self::Target {
    self.as_bytes()
  }
}

impl DnsFrame<'_> {
  pub fn header(&self) -> &DnsHeader {
    unsafe { &*(self.buf[..HEADER_SIZE].as_ptr() as *const _ as *const DnsHeader) }
  }

  pub fn header_mut(&mut self) -> &mut DnsHeader {
    unsafe { &mut *(self.buf[..HEADER_SIZE].as_mut_ptr() as *mut _ as *mut DnsHeader) }
  }

  pub fn add_question(&mut self, question: &Question) {
    self.extend(&question.as_bytes());
  }

  pub fn add_ttl(&mut self, ttl: u32) {
    self.extend(&ttl.to_be_bytes());
  }

  pub fn add_rdata(&mut self, data: &[u8]) {
    self.extend(&(data.len() as u16).to_be_bytes());
    self.extend(data);
  }

  fn extend(&mut self, bytes: &[u8]) {
    self.buf[self.len..(self.len + bytes.len())].copy_from_slice(bytes);
    self.len += bytes.len();
  }

  pub fn as_bytes(&self) -> &[u8] {
    &self.buf[..self.len]
  }

  pub fn questions(&self) -> Questions {
    Questions {
      question_count: self.header().question_count() as usize,
      current_question: 0,
      buf: &self.as_bytes(),
      buf_i: HEADER_SIZE,
    }
  }
}
