use core::fmt;
use core::str;

/// A DNS name.
#[derive(Debug)]
pub struct Name<'a> {
  pub(crate) buf: &'a [u8],
  pub(crate) start: usize,
  pub(crate) end: usize,
}

pub(crate) const fn is_pointer(len: u8) -> bool {
  (len >> 6) == 0b11
}

pub(crate) const fn mask_len(len: u8) -> u8 {
  len & 0b00111111
}

impl fmt::Display for Name<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut i = self.start;
    let mut print_dot = false;

    loop {
      match read_label(self.buf, &mut i) {
        None => return Err(fmt::Error),
        Some(Label::Pointer(ptr)) => {
          i = ptr as usize;
          continue;
        },
        Some(Label::Part(len)) => {
          if len == 0 {
            return Ok(())
          }

          if print_dot {
            '.'.fmt(f)?;
          }

          for &c in &self.buf[(i - len as usize)..i] {
            (c as char).fmt(f)?;
          }

          print_dot = true;
        },
      }
    }
  }
}

impl PartialEq<&str> for Name<'_> {
  fn eq(&self, other: &&str) -> bool {
    let mut i = self.start;
    let mut other_i = 0;

    let other = other.as_bytes();

    loop {
      match read_label(self.buf, &mut i) {
        None => return false,
        Some(Label::Pointer(ptr)) => {
          i = ptr as usize;
          continue;
        },
        Some(Label::Part(len)) => {
          if len == 0 {
            return other_i == other.len()
          }

          if other_i != 0 {
            if other.get(other_i) != Some(&b'.') {
              return false
            } else {
              other_i += 1;
            }
          }

          if let Some(substring) = other.get(other_i..(other_i + len as usize)) {
            if !self.buf[(i - len as usize)..i].eq_ignore_ascii_case(substring) {
              return false
            }
          } else {
            return false
          }

          other_i += len as usize;
        },
      }
    }
  }
}

pub(crate) enum Label {
  Pointer(u16),
  Part(u8),
}

pub(crate) fn read_name(buf: &[u8], i: &mut usize) -> bool {
  loop {
    match read_label(buf, i) {
      None => return false,
      Some(Label::Pointer(ptr)) => {
        return read_name(buf, &mut (ptr as usize));
      },
      Some(Label::Part(len)) => {
        if len == 0 {
          return true
        }
      },
    }
  }
}

// Return whether a label was read and whether it was the last label.
pub(crate) fn read_label(buf: &[u8], i: &mut usize) -> Option<Label> {
  if let Some(&pointer_or_len) = buf.get(*i) {
    let len = mask_len(pointer_or_len);

    // Check for pointer:
    // https://tools.ietf.org/rfc/rfc1035#section-4.1.4
    if is_pointer(pointer_or_len) {
      if let Some(&len2) = buf.get(*i + 1) {
        *i += 1 + 1;
        Some(Label::Pointer(((len as u16) << 8) + len2 as u16))
      } else {
        None
      }
    } else {
      *i += 1 + len as usize;
      Some(Label::Part(len))
    }
  } else {
    None
  }
}
