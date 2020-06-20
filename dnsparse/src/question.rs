use core::fmt;
use core::str;

use crate::{ResponseCode, QueryKind, QueryClass};

#[repr(C)]
pub struct Question {
  buf: [u8; 256],
  len: usize,
}

impl fmt::Debug for Question {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    fmt.debug_struct("Question")
      .field("name", &self.name())
      .field("kind", &self.kind())
      .field("class", &self.class())
      .finish()
  }
}

impl Question {
  pub fn name(&self) -> &str {
    unsafe { str::from_utf8_unchecked(&self.buf[1..(self.len - 5)]) }
  }

  pub fn kind(&self) -> QueryKind {
    let b0 = self.len - 4;
    let b1 = b0 + 1;
    QueryKind::from(u16::from_be_bytes([self.buf[b0], self.buf[b1]]))
  }

  pub fn class(&self) -> QueryClass {
    let b0 = self.len - 2;
    let b1 = b0 + 1;
    QueryClass::from(u16::from_be_bytes([self.buf[b0], self.buf[b1]]))
  }

  pub fn as_bytes(&self) -> &[u8] {
    &self.buf[..self.len]
  }
}

pub struct Questions<'a> {
  pub(crate) question_count: usize,
  pub(crate) current_question: usize,
  pub(crate) buf: &'a [u8],
  pub(crate) buf_i: usize,
}

impl<'a> Iterator for Questions<'a> {
  type Item = Result<Question, ResponseCode>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current_question >= self.question_count {
      return None
    }

    let mut buf = [0; 256];
    let mut len = 0;

    loop {
      let i = self.buf_i + len;

      let part_len = if let Some(&len) = self.buf.get(i) {
        len as usize
      } else {
        return Some(Err(ResponseCode::FormatError))
      };

      if part_len == 0 {
        if let Some(part) = self.buf.get(i..(i + 5)) {
          buf[len..(len + 5)].copy_from_slice(part);
          len += 5;
        } else {
          return Some(Err(ResponseCode::FormatError))
        };

        self.current_question += 1;
        self.buf_i += len;

        return Some(Ok(Question { buf, len }))
      } else {
        buf[len] = '.' as u32 as u8;

        if let Some(part) = self.buf.get((i + 1)..(i + 1 + part_len)) {
          buf[(len + 1)..(len + 1 + part_len)].copy_from_slice(part);
        } else {
          return Some(Err(ResponseCode::FormatError))
        }
      }

      len += part_len + 1;
    }
  }
}

