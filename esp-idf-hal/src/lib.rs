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

/// Enumeration of all available interfaces.
#[derive(Debug, Clone, Copy)]
pub enum Interface {
  /// WiFi interface in station mode.
  Sta,
  /// WiFi interface in access point mode.
  Ap,
  #[cfg(not(target_device = "esp8266"))]
  /// Bluetooth interface.
  Bt,
  /// Ethernet interface.
  #[cfg(not(target_device = "esp8266"))]
  Eth,
}

/// ```no_run
/// use macaddr::MacAddr6;
/// use esp32_hal::Interface;
///
/// MacAddr6::from(Interface::Ap)
/// ```
impl From<Interface> for MacAddr6 {
  fn from(interface: Interface) -> Self {
    let mac_address_type = match interface {
      Interface::Sta => esp_mac_type_t::ESP_MAC_WIFI_STA,
      Interface::Ap  => esp_mac_type_t::ESP_MAC_WIFI_SOFTAP,
      #[cfg(not(target_device = "esp8266"))]
      Interface::Bt  => esp_mac_type_t::ESP_MAC_BT,
      #[cfg(not(target_device = "esp8266"))]
      Interface::Eth => esp_mac_type_t::ESP_MAC_ETH,
    };

    let mut mac_address = MaybeUninit::<Self>::uninit();
    assert_esp_ok!(esp_read_mac(mac_address.as_mut_ptr() as *mut _, mac_address_type));
    unsafe { mac_address.assume_init() }
  }
}

/// ```no_run
/// use macaddr::MacAddr;
/// use esp32_hal::Interface;
///
/// MacAddr::from(Interface::Ap)
/// ```
impl From<Interface> for MacAddr {
  fn from(interface: Interface) -> Self {
    Self::V6(interface.into())
  }
}
