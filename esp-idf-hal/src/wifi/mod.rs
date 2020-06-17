use core::marker::PhantomData;
use core::mem::transmute;
use core::str::Utf8Error;
use core::ptr;

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;

use core::fmt;

use crate::{EspError, nvs::NonVolatileStorage, hprintln};

use esp_idf_bindgen::{
  libc,
  ESP_ERR_INVALID_STATE,
  esp_event_loop_create_default,
  esp_err_t,
  wifi_mode_t,
  wifi_ap_config_t,
  esp_wifi_get_mode,

  esp_wifi_stop,
  esp_wifi_start,

  esp_wifi_init,
  esp_wifi_scan_start,
  esp_wifi_set_mode,
  esp_wifi_set_config,

  wifi_scan_type_t,
  wifi_scan_config_t,
  wifi_sta_config_t,
  wifi_scan_threshold_t,
  wifi_scan_method_t,
  wifi_sort_method_t,
  wifi_auth_mode_t,
  wifi_err_reason_t,

  ESP_ERR_WIFI_NOT_INIT,

  wifi_init_config_t,
  wifi_scan_time_t,
  wifi_active_scan_time_t,
  esp_interface_t,
  wifi_config_t,
  esp_wifi_connect,
  esp_wifi_scan_get_ap_num,
  wifi_ap_record_t,
  esp_wifi_scan_get_ap_records,
  esp_netif_init,
  wifi_pmf_config_t,
};

mod sta_config;
pub use sta_config::*;

mod ap_config;
pub use ap_config::*;

const SSID_MAX_LEN: usize = 32;
const PASSWORD_MAX_LEN: usize = 64;

#[derive(Debug)]
pub enum WifiConfigError {
  InteriorNul(usize),
  TooLong(usize, usize),
  Utf8Error(Utf8Error),
}

impl fmt::Display for WifiConfigError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InteriorNul(pos) => write!(f, "data provided contains an interior nul byte at pos {}", pos),
      Self::TooLong(max, actual) => write!(f, "data provided is {} bytes long, but maximum is {} bytes", max, actual),
      Self::Utf8Error(utf8_error) => utf8_error.fmt(f),
    }
  }
}

#[derive(Clone)]
pub struct Ssid {
  ssid: [u8; SSID_MAX_LEN],
  ssid_len: usize,
}

impl Ssid {
  pub fn as_str(&self) -> &str {
    &unsafe { core::str::from_utf8_unchecked(&self.ssid[..self.ssid_len]) }
  }

  pub fn from_bytes(bytes: &[u8]) -> Result<Ssid, WifiConfigError> {
    let ssid_len = bytes.len();

    if ssid_len > SSID_MAX_LEN {
      return Err(WifiConfigError::TooLong(SSID_MAX_LEN, ssid_len))
    }

    if let Err(utf8_error) = core::str::from_utf8(bytes) {
      return Err(WifiConfigError::Utf8Error(utf8_error))
    }

    if let Some(pos) = memchr::memchr(0, bytes) {
      return Err(WifiConfigError::InteriorNul(pos))
    }

    Ok(unsafe { Self::from_bytes_unchecked(bytes) })
  }

  pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> Ssid {
    let ssid_len = bytes.len();
    let mut ssid = [0; SSID_MAX_LEN];
    ptr::copy_nonoverlapping(bytes.as_ptr(), ssid.as_mut_ptr(), ssid_len);
    Self { ssid, ssid_len }
  }
}

impl fmt::Debug for Ssid {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Ssid")
      .field("ssid", &self.as_str())
      .field("ssid_len", &self.ssid_len)
      .finish()
  }
}

impl fmt::Display for Ssid {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.as_str().fmt(f)
  }
}

#[derive(Clone)]
pub struct Password {
  password: [u8; PASSWORD_MAX_LEN],
  password_len: usize,
}

impl Password {
  pub fn as_str(&self) -> &str {
    &unsafe { core::str::from_utf8_unchecked(&self.password[..self.password_len]) }
  }

  pub fn from_bytes(bytes: &[u8]) -> Result<Password, WifiConfigError> {
    let ssid_len = bytes.len();

    if ssid_len > PASSWORD_MAX_LEN {
      return Err(WifiConfigError::TooLong(PASSWORD_MAX_LEN, ssid_len))
    }

    if let Err(utf8_error) = core::str::from_utf8(bytes) {
      return Err(WifiConfigError::Utf8Error(utf8_error))
    }

    if let Some(pos) = memchr::memchr(0, bytes) {
      return Err(WifiConfigError::InteriorNul(pos))
    }

    Ok(unsafe { Self::from_bytes_unchecked(bytes) })
  }

  pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> Password {
    let password_len = bytes.len();
    let mut password = [0; PASSWORD_MAX_LEN];
    ptr::copy_nonoverlapping(bytes.as_ptr(), password.as_mut_ptr(), password_len);
    Self { password, password_len }
  }
}

impl Default for Password {
  fn default() -> Self {
    Self { password: [0; PASSWORD_MAX_LEN], password_len: 0 }
  }
}

impl fmt::Debug for Password {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Password")
      .field("password", &"********")
      .field("password_len", &8)
      .finish()
  }
}

impl fmt::Display for Password {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    #[cfg(debug)]
    return self.as_str().fmt(f);

    #[cfg(not(debug))]
    return "********".fmt(f);
  }
}

#[derive(Debug, Clone, Copy)]
pub enum AuthMode {
  Open,
  Wpa2Psk,
}

impl From<AuthMode> for wifi_auth_mode_t {
  fn from(auth_mode: AuthMode) -> wifi_auth_mode_t {
    match auth_mode {
      AuthMode::Open => wifi_auth_mode_t::WIFI_AUTH_OPEN,
      AuthMode::Wpa2Psk => wifi_auth_mode_t::WIFI_AUTH_WPA2_PSK,
    }
  }
}

pub fn netif_init() -> Result<(), EspError> {
  EspError::result(unsafe { esp_netif_init() })
}

pub fn wifi_init(nvs: &mut NonVolatileStorage) -> Result<(), EspError> {
  nvs.flash_init()?;
  let config = wifi_init_config_t::default();
  EspError::result(unsafe { esp_wifi_init(&config) })
}

pub fn wifi_start() -> Result<(), EspError> {
  EspError::result(unsafe { esp_wifi_start() })
}

pub fn wifi_stop() -> Result<(), EspError> {
  EspError::result(unsafe { esp_wifi_stop() })
}

pub fn get_mode() -> Result<wifi_mode_t, EspError> {
  let mut mode = wifi_mode_t::WIFI_MODE_NULL;

  match EspError::result(unsafe { esp_wifi_get_mode(&mut mode) }) {
    Ok(()) => Ok(mode),
    Err(err) if err.code == ESP_ERR_WIFI_NOT_INIT as esp_err_t => {
      Ok(wifi_mode_t::WIFI_MODE_NULL)
    },
    Err(err) => Err(err)
  }
}

pub fn set_mode(mode: wifi_mode_t) -> Result<(), EspError> {
  EspError::result(unsafe { esp_wifi_set_mode(mode) })
}

pub fn set_config(interface: esp_interface_t, config: &mut wifi_config_t) -> Result<(), EspError> {
  EspError::result(unsafe { esp_wifi_set_config(interface, config) })
}

pub fn wifi_scan(show_hidden: bool, passive: bool, max_ms_per_channel: u32) -> Result<(), EspError> {
  let mut config = wifi_scan_config_t {
    ssid: core::ptr::null_mut(),
    bssid: core::ptr::null_mut(),
    channel: 0,
    show_hidden,
    scan_type: wifi_scan_type_t::WIFI_SCAN_TYPE_ACTIVE,
    scan_time: wifi_scan_time_t {
      active: wifi_active_scan_time_t { min: 0, max: 0 },
      passive: 0,
    },
  };

  if passive {
    config.scan_type = wifi_scan_type_t::WIFI_SCAN_TYPE_PASSIVE;
    config.scan_time.passive = max_ms_per_channel;
  } else {
    config.scan_type = wifi_scan_type_t::WIFI_SCAN_TYPE_ACTIVE;
    config.scan_time.active = wifi_active_scan_time_t { min: 100, max: max_ms_per_channel };
  }

  EspError::result(unsafe { esp_wifi_scan_start(&config, false) })
}

pub struct Wifi<T = ()> {
  mode: PhantomData<T>
}

impl Wifi {
  pub fn init(nvs: &mut NonVolatileStorage) -> Result<Wifi<()>, EspError> {
    netif_init()?;

    match event_loop_create_default() {
      Ok(()) => (),
      Err(err) if err.code == ESP_ERR_INVALID_STATE as _ => (),
      err => err.unwrap(),
    }

    unsafe { esp_idf_bindgen::esp_netif_create_default_wifi_ap() };
    unsafe { esp_idf_bindgen::esp_netif_create_default_wifi_sta() };

    wifi_init(nvs)?;

    Ok(Wifi { mode: PhantomData })
  }

  pub fn into_ap(self, config: &ApConfig) -> Result<Wifi<Ap>, EspError> {

    let mut config = wifi_config_t::from(config);

    set_mode(wifi_mode_t::WIFI_MODE_AP)?;
    set_config(esp_interface_t::ESP_IF_WIFI_AP, &mut config)?;

    Ok(Wifi { mode: PhantomData })
  }

  pub fn into_sta(self, config: &StaConfig) -> Result<Wifi<Sta>, EspError> {
    let mut config = wifi_config_t::from(config);

    set_mode(wifi_mode_t::WIFI_MODE_STA)?;
    set_config(esp_interface_t::ESP_IF_WIFI_STA, &mut config)?;

    Ok(Wifi { mode: PhantomData })
  }
}

#[cfg(target_device = "esp32")]
unsafe extern "C" fn wifi_scan_done_handler(
  _event_handler_arg: *mut libc::c_void,
  _event_base: esp_idf_bindgen::esp_event_base_t,
  _event_id: i32,
  _event_data: *mut libc::c_void,
) {
  let mut ap_num = 0;
  EspError::result(esp_wifi_scan_get_ap_num(&mut ap_num)).unwrap();

  hprintln!("Found {} APs:", ap_num);

  use core::mem::MaybeUninit;

  let mut aps: Vec<MaybeUninit<wifi_ap_record_t>> = vec![MaybeUninit::uninit().assume_init(); ap_num as usize];
  if EspError::result(esp_wifi_scan_get_ap_records(&mut ap_num, aps.as_mut_ptr() as *mut wifi_ap_record_t)).is_ok() {

    for i in 0..ap_num {
      let ap = aps[i as usize].assume_init();

      let ssid = ap.ssid.iter().take_while(|&c| *c != 0).map(|&c| core::char::from_u32_unchecked(c as u32)).collect::<String>();
      hprintln!("- {}: {}", i, ssid);
    }
  }
}

pub struct Sta;
pub struct Ap;



#[must_use = "WiFi will be stopped immediately. Drop it explicitly after you are done using it or create a named binding."]
#[derive(Debug)]
pub struct StaRunning(pub [u8; 4]);

#[must_use = "WiFi will be stopped immediately. Drop it explicitly after you are done using it or create a named binding."]
#[derive(Debug)]
pub struct ApRunning;

impl ApRunning {
  pub fn stop(self) -> Wifi<()> {
    Wifi { mode: PhantomData }
  }
}

impl StaRunning {
  pub fn stop(self) -> Wifi<()> {
    Wifi { mode: PhantomData }
  }
}

impl Drop for ApRunning {
  fn drop(&mut self) {
    let _ = wifi_stop();
  }
}

impl Drop for StaRunning {
  fn drop(&mut self) {
    let _ = wifi_stop();
  }
}


impl Wifi<Ap> {
  pub fn start(self) -> ApRunning {
    wifi_start().unwrap();
    ApRunning
  }
}

impl Wifi<Sta> {
  pub fn connect(self) -> ConnectFuture {
    ConnectFuture { state: ConnectFutureState::Starting }
  }
}

#[derive(Debug)]
enum ConnectFutureState {
  Failed(WifiError),
  Starting,
  Connected([u8; 4]),
}

#[must_use = "futures do nothing unless polled"]
pub struct ConnectFuture {
  state: ConnectFutureState,
}

use core::task::{Poll, Context, Waker};
use core::pin::Pin;

#[derive(Debug, Clone)]
pub enum WifiError {
  Internal(EspError),
  ConnectionError(wifi_err_reason_t),
}

impl WifiError {
  pub fn wifi(self) -> Wifi<()> {
    Wifi { mode: PhantomData }
  }
}

impl From<EspError> for WifiError {
  fn from(esp_error: EspError) -> Self {
    Self::Internal(esp_error)
  }
}

impl fmt::Display for WifiError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Internal(esp_error) => esp_error.fmt(f),
      Self::ConnectionError(reason) => fmt::Debug::fmt(reason, f),
    }
  }
}

impl core::future::Future for ConnectFuture {
  type Output = Result<StaRunning, WifiError>;

  #[cfg(target_device = "esp8266")]
  fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    Poll::Pending
  }

  #[cfg(target_device = "esp32")]
  fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    use esp_idf_bindgen::{esp_event_handler_register, esp_event_handler_unregister, ip_event_t, wifi_event_t, IP_EVENT, WIFI_EVENT};

    match &self.state {
      ConnectFutureState::Starting => {
        let b1: Box<(&ConnectFuture, Waker)> = Box::new((&self, cx.waker().clone()));
        let b2: Box<(&ConnectFuture, Waker)> = Box::new((&self, cx.waker().clone()));
        let b3: Box<(&ConnectFuture, Waker)> = Box::new((&self, cx.waker().clone()));
        let b4: Box<(&ConnectFuture, Waker)> = Box::new((&self, cx.waker().clone()));

        EspError::result(unsafe {
          esp_event_handler_register(WIFI_EVENT, wifi_event_t::WIFI_EVENT_STA_START as _, Some(wifi_sta_handler), Box::into_raw(b1) as *mut _)
        })?;
        EspError::result(unsafe {
          esp_event_handler_register(WIFI_EVENT, wifi_event_t::WIFI_EVENT_STA_CONNECTED as _, Some(wifi_sta_handler), Box::into_raw(b2) as *mut _)
        })?;
        EspError::result(unsafe {
          esp_event_handler_register(WIFI_EVENT, wifi_event_t::WIFI_EVENT_STA_DISCONNECTED as _, Some(wifi_sta_handler), Box::into_raw(b3) as *mut _)
        })?;
        EspError::result(unsafe {
          esp_event_handler_register(IP_EVENT, ip_event_t::IP_EVENT_STA_GOT_IP as _, Some(wifi_sta_handler), Box::into_raw(b4) as *mut _)
        })?;
        wifi_start()?;
        Poll::Pending
      },
      state => {
        EspError::result(unsafe { esp_event_handler_unregister(WIFI_EVENT, wifi_event_t::WIFI_EVENT_STA_START as _, Some(wifi_sta_handler)) })?;
        EspError::result(unsafe { esp_event_handler_unregister(WIFI_EVENT, wifi_event_t::WIFI_EVENT_STA_CONNECTED as _, Some(wifi_sta_handler)) })?;
        EspError::result(unsafe { esp_event_handler_unregister(WIFI_EVENT, wifi_event_t::WIFI_EVENT_STA_DISCONNECTED as _, Some(wifi_sta_handler)) })?;
        EspError::result(unsafe { esp_event_handler_unregister(IP_EVENT, ip_event_t::IP_EVENT_STA_GOT_IP as _, Some(wifi_sta_handler)) })?;

        match *state {
          ConnectFutureState::Starting => unreachable!(),
          ConnectFutureState::Failed(ref err) => {
            let _ = wifi_stop();
            Poll::Ready(Err(err.clone().into()))
          },
          ConnectFutureState::Connected(ip) => {
            Poll::Ready(Ok(StaRunning(ip)))
          }
        }
      }
    }
  }
}

#[cfg(target_device = "esp32")]
unsafe extern "C" fn wifi_sta_handler(
  event_handler_arg: *mut libc::c_void,
  event_base: esp_idf_bindgen::esp_event_base_t,
  event_id: i32,
  event_data: *mut libc::c_void,
) {
  use esp_idf_bindgen::{ip_event_t, ip_event_got_ip_t, wifi_event_sta_connected_t, wifi_event_sta_disconnected_t, wifi_event_t, IP_EVENT, WIFI_EVENT};

  let b = Box::from_raw(event_handler_arg as *mut (&mut ConnectFuture, Waker));
  let (mut f, waker) = *b;

  if event_base == WIFI_EVENT {
    let event_id: wifi_event_t = transmute(event_id);

    hprintln!("WIFI_EVENT: {:?}", event_id);

    match event_id {
      wifi_event_t::WIFI_EVENT_STA_START => {
        let res = EspError::result(esp_wifi_connect());

        if let Err(err) = res {
          f.state = ConnectFutureState::Failed(err.into());
          waker.wake();
        }
      },
      wifi_event_t::WIFI_EVENT_STA_CONNECTED => {
        let data = *(event_data as *mut wifi_event_sta_connected_t);
        hprintln!("EVENT_DATA: {:?}", data);
      },
      wifi_event_t::WIFI_EVENT_STA_DISCONNECTED => {
        let data = *(event_data as *mut wifi_event_sta_disconnected_t);
        let reason: wifi_err_reason_t = transmute(data.reason as u32);

        hprintln!("EVENT_DATA: {:?}", data);

        f.state = ConnectFutureState::Failed(WifiError::ConnectionError(reason));
        waker.wake();
      },
      _ => (),
    }
  } else if event_base == IP_EVENT {
    let event_id: ip_event_t = transmute(event_id);

    hprintln!("IP_EVENT: {:?}", event_id);

    match event_id {
      ip_event_t::IP_EVENT_STA_GOT_IP => {
        let event: ip_event_got_ip_t = *(event_data as *mut ip_event_got_ip_t);
        let octets = u32::from_be(event.ip_info.ip.addr).to_be_bytes();
        f.state = ConnectFutureState::Connected(octets);
        waker.wake();
      },
      _ => (),
    }
  }
}

pub fn event_loop_create_default() -> Result<(), EspError> {
  EspError::result(unsafe { esp_event_loop_create_default() })
}
