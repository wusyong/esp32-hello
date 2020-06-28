use core::fmt;
use core::mem::size_of;

use crate::{Name, QueryKind, QueryClass};
use crate::name::read_name;

/// A DNS question.
#[repr(C)]
pub struct Question<'a> {
  buf: &'a [u8],
  start: usize,
  end: usize,
}

impl fmt::Debug for Question<'_> {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    fmt.debug_struct("Question")
      .field("name", &self.name())
      .field("kind", &self.kind())
      .field("class", &self.class())
      .finish()
  }
}

impl<'a> Question<'a> {
  pub fn name(&self) -> Name<'a> {
    Name { buf: self.buf, start: self.start, end: self.end - 5 }
  }

  pub fn kind(&self) -> QueryKind {
    let b0 = self.end - 4;
    let b1 = b0 + 1;
    QueryKind::from(u16::from_be_bytes([self.buf[b0], self.buf[b1]]))
  }

  pub fn class(&self) -> QueryClass {
    let b0 = self.end - 2;
    let b1 = b0 + 1;
    QueryClass::from(u16::from_be_bytes([self.buf[b0], self.buf[b1]]))
  }

  pub fn as_bytes(&self) -> &'a [u8] {
    &self.buf[self.start..self.end]
  }
}

/// Iterator over [`Question`](struct.Question.html)s contained in a [`DnsFrame`](struct.DnsFrame.html).
pub struct Questions<'a> {
  pub(crate) question_count: usize,
  pub(crate) current_question: usize,
  pub(crate) buf: &'a [u8],
  pub(crate) buf_i: usize,
}

pub(crate) fn read_question(buf: &[u8], i: &mut usize) -> bool {
  if read_name(buf, i) {
    read_query_class_and_kind(buf, i)
  } else {
    false
  }
}

#[inline]
fn read_query_class_and_kind(buf: &[u8], i: &mut usize) -> bool {
  if *i + size_of::<QueryClass>() + size_of::<QueryKind>() <= buf.len() {
    *i += size_of::<QueryClass>() + size_of::<QueryKind>();
    true
  } else {
    false
  }
}

impl<'a> Iterator for Questions<'a> {
  type Item = Question<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current_question >= self.question_count {
      return None
    }

    let mut i = self.buf_i;

    assert!(read_question(&self.buf, &mut i));
    let question = Question { buf: &self.buf, start: self.buf_i, end: i };

    self.current_question += 1;
    self.buf_i = i;

    return Some(question)
  }
}

