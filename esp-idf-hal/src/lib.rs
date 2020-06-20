#![feature(never_type)]
#![warn(missing_debug_implementations)]

use core::mem::MaybeUninit;
use std::ffi::CStr;
use core::str;

#[macro_use]
extern crate alloc;

use macaddr::{MacAddr, MacAddr6};

use esp_idf_bindgen::{esp_err_t, esp_mac_type_t, esp_err_to_name, esp_read_mac};

pub mod ets;
pub mod netif;
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

#[derive(Debug, Clone, Copy)]
pub enum MacAddrType {
  /// Mac address type of the WiFi interface in station mode.
  Sta,
  /// Mac address type of the WiFi interface in access point mode.
  Ap,
  #[cfg(not(target_device = "esp8266"))]
  /// Mac address type of the Bluetooth interface.
  Bt,
  /// Mac address type of the Ethernet interface.
  #[cfg(not(target_device = "esp8266"))]
  Eth,
}

/// ```no_run
/// use macaddr::MacAddr6;
/// use esp32_hal::MacAddrType;
///
/// MacAddr6::from(MacAddrType::Ap)
/// ```
impl From<MacAddrType> for MacAddr6 {
  fn from(mac_address_type: MacAddrType) -> Self {
    let mac_address_type = match mac_address_type {
      MacAddrType::Sta => esp_mac_type_t::ESP_MAC_WIFI_STA,
      MacAddrType::Ap  => esp_mac_type_t::ESP_MAC_WIFI_SOFTAP,
      #[cfg(not(target_device = "esp8266"))]
      MacAddrType::Bt  => esp_mac_type_t::ESP_MAC_BT,
      #[cfg(not(target_device = "esp8266"))]
      MacAddrType::Eth => esp_mac_type_t::ESP_MAC_ETH,
    };

    let mut mac_address = MaybeUninit::<Self>::uninit();
    assert_esp_ok!(esp_read_mac(mac_address.as_mut_ptr() as *mut _, mac_address_type));
    unsafe { mac_address.assume_init() }
  }
}

/// ```no_run
/// use macaddr::MacAddr;
/// use esp32_hal::MacAddrType;
///
/// MacAddr::from(MacAddrType::Ap)
/// ```
impl From<MacAddrType> for MacAddr {
  fn from(mac_address_type: MacAddrType) -> Self {
    Self::V6(mac_address_type.into())
  }
}
