use embedded_hal::blocking::delay::{DelayMs, DelayUs};

extern "C" {
  fn ets_delay_us(us: u32);
  fn ets_printf(fmt: *const i8, ...) -> i32;
}

#[derive(Debug)]
pub struct Ets;

impl DelayUs<u8> for Ets {
  fn delay_us(&mut self, us: u8) {
    unsafe { ets_delay_us(u32::from(us)) };
  }
}

impl DelayUs<u16> for Ets {
  fn delay_us(&mut self, us: u16) {
    unsafe { ets_delay_us(u32::from(us)) };
  }
}

impl DelayUs<u32> for Ets {
  fn delay_us(&mut self, us: u32) {
    unsafe { ets_delay_us(us) };
  }
}

impl DelayMs<u8> for Ets {
  fn delay_ms(&mut self, ms: u8) {
    let us = u32::from(ms) * 1000;
    unsafe { ets_delay_us(us) };
  }
}

impl DelayMs<u16> for Ets {
  fn delay_ms(&mut self, ms: u16) {
    let us = u32::from(ms) * 1000;
    unsafe { ets_delay_us(us) };
  }
}

impl DelayMs<u32> for Ets {
  fn delay_ms(&mut self, ms: u32) {
    let us = ms * 1000;
    unsafe { ets_delay_us(us) };
  }
}

/// ```
/// use core::fmt::Write;
///
/// let mut ets = Ets;
///
/// writeln!(ets, "Hello, world!");
/// ```
impl core::fmt::Write for Ets {
  fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
    for c in s.chars() {
      self.write_char(c)?;
    }

    Ok(())
  }

  fn write_char(&mut self, c: char) -> Result<(), core::fmt::Error> {
    unsafe { ets_printf("%c\0".as_ptr() as *const i8, c as i32) };
    Ok(())
  }
}
