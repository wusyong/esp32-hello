#![feature(never_type)]
#![feature(const_cstr_unchecked)]
#![warn(missing_debug_implementations)]

use std::ffi::CStr;
use std::str;

#[macro_use]
extern crate alloc;

use esp_idf_bindgen::{esp_err_t, esp_err_to_name};

pub mod ets;
pub mod interface;
mod heap;
pub use heap::Heap;
pub mod wifi;
pub mod nvs;

#[derive(Clone, Debug)]
pub struct EspError { code: esp_err_t }

#[macro_export]
macro_rules! esp_ok {
  ($err:expr) => {{
    let code = unsafe { $err };
    if code == ::esp_idf_bindgen::ESP_OK as ::esp_idf_bindgen::esp_err_t {
      Ok(())
    } else {
      Err($crate::EspError { code })
    }
  }}
}

#[macro_export]
macro_rules! assert_esp_ok {
  ($err:expr) => {
    if let Err(err) = $crate::esp_ok!($err) {
      panic!("assertion failed: `{} == ESP_OK`", err);
    }
  }
}

impl From<!> for EspError {
  fn from(_: !) -> Self {
    loop {}
  }
}

impl core::fmt::Display for EspError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    unsafe {
      let s = CStr::from_ptr(esp_err_to_name(self.code));
      str::from_utf8_unchecked(s.to_bytes()).fmt(f)
    }
  }
}
