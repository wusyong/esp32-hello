use std::ffi::CStr;
use std::str;

use esp_idf_bindgen::{esp_err_t, esp_err_to_name};

#[derive(Clone, Debug)]
pub struct EspError { pub(crate) code: esp_err_t }


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

macro_rules! esp_ok {
  ($err:expr) => {{
    let code = unsafe { $err };
    if code == ::esp_idf_bindgen::ESP_OK as ::esp_idf_bindgen::esp_err_t {
      Ok(())
    } else {
      Err($crate::esp_error::EspError { code })
    }
  }}
}
