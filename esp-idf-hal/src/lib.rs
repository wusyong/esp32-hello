#![feature(never_type)]
#![warn(missing_debug_implementations)]
#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt::Write;
use core::mem::MaybeUninit;

#[macro_use]
extern crate alloc;

use macaddr::{MacAddr, MacAddr6};

use esp_idf_bindgen::{esp_err_t, esp_mac_type_t, ESP_OK, esp_err_to_name, esp_read_mac};

pub mod ets;
pub mod netif;
pub mod wifi;
pub mod nvs;

#[cfg(feature = "panic_handler")]
#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
  hprintln!("{}", panic_info);

  unsafe {
    abort();
    core::hint::unreachable_unchecked()
  }
}

#[macro_export]
macro_rules! hprint {
  ($($s:expr),*) => {{
    use core::fmt::Write;
    write!(crate::ets::Ets, $($s),*).unwrap();
  }}
}

#[macro_export]
macro_rules! hprintln {
  ($($s:expr),*) => {{
    use core::fmt::Write;
    writeln!(crate::ets::Ets, $($s),*).unwrap();
  }}
}

#[macro_export]
macro_rules! cstring {
  ($s:expr) => {{
    let mut name: Vec<libc::c_char> = $s.chars().map(|c| c as libc::c_char).collect();
    name.push(0);
    name
  }}
}

#[macro_export]
macro_rules! ptr_set_mask {
  ($register:expr, $mask:expr) => {
    let ptr = $register as *mut u32;
    core::ptr::write_volatile(ptr, core::ptr::read_volatile(ptr) | ($mask));
  };
}

#[macro_export]
macro_rules! ptr_clear_mask {
  ($register:expr, $mask:expr) => {
    let ptr = $register as *mut u32;
    core::ptr::write_volatile(ptr, core::ptr::read_volatile(ptr) & !($mask));
  };
}

#[derive(Clone, Debug)]
pub struct EspError { code: esp_err_t }

impl EspError {
  pub fn result(code: esp_err_t) -> Result<(), Self> {
    if code == ESP_OK as esp_err_t {
      return Ok(())
    } else {
      Err(EspError { code })
    }
  }
}

impl From<!> for EspError {
  fn from(_: !) -> Self {
    loop {}
  }
}

impl From<esp_err_t> for EspError {
  fn from(code: esp_err_t) -> Self {
    EspError { code }
  }
}

impl core::fmt::Display for EspError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    unsafe {
      let mut name = esp_err_to_name(self.code);

      while !name.is_null() {
        let c = core::char::from_u32_unchecked(*name as u32);

        if c == '\0' { break }
        f.write_char(c)?;
        name = name.add(1);
      }
    }

    Ok(())
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
    unsafe {
      let err = esp_read_mac(mac_address.as_mut_ptr() as *mut _, mac_address_type);
      debug_assert_eq!(err, ESP_OK as _);
      mac_address.assume_init()
    }
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
