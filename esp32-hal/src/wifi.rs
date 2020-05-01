use core::ptr;
use core::marker::PhantomData;
use core::mem::transmute;

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;

use static_assertions::assert_eq_size;

use crate::{EspError, nvs::NonVolatileStorage, hprintln};

use esp_idf_sys::{
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
  pub fn init(nvs: &mut NonVolatileStorage) -> Wifi<()> {
    netif_init().unwrap();

    match event_loop_create_default() {
      Ok(()) => (),
      Err(err) if err.code == ESP_ERR_INVALID_STATE as _ => (),
      err => err.unwrap(),
    }
    wifi_init(nvs).unwrap();

    Wifi { mode: PhantomData }
  }

  pub fn into_ap(self, config: &ApConfig) -> Wifi<Ap> {
    let ssid_len = config.ssid.iter().take_while(|&&c| c != 0).count();

    let mut config = wifi_config_t {
      ap: wifi_ap_config_t {
        ssid: config.ssid,
        ssid_len: ssid_len as u8,
        password: config.password,
        channel: config.channel,
        authmode: config.auth_mode.into(),
        ssid_hidden: config.ssid_hidden as u8,
        max_connection: config.max_connection,
        beacon_interval: config.beacon_interval,
      },
    };

    set_mode(wifi_mode_t::WIFI_MODE_AP).unwrap();
    set_config(esp_interface_t::ESP_IF_WIFI_AP, &mut config).unwrap();

    Wifi { mode: PhantomData }
  }

  pub fn into_sta(self, config: &StaConfig) -> Wifi<Sta> {
    let scan_method = match config.scan_method {
      ScanMethod::Fast => wifi_scan_method_t::WIFI_FAST_SCAN,
      ScanMethod::Full => wifi_scan_method_t::WIFI_ALL_CHANNEL_SCAN,
    };

    let bssid = if let Some(bssid) = config.bssid {
      bssid
    } else {
      [0, 0, 0, 0, 0, 0]
    };

    let sort_method = match config.sort_method {
      SortMethod::BySignal => wifi_sort_method_t::WIFI_CONNECT_AP_BY_SIGNAL,
    };

    let threshold = if let Some(threshold) = config.threshold {
      wifi_scan_threshold_t {
        rssi: threshold.rssi,
        authmode: threshold.auth_mode.into(),
      }
    } else {
      wifi_scan_threshold_t {
        rssi: -127,
        authmode: wifi_auth_mode_t::WIFI_AUTH_OPEN,
      }
    };

    let pmf_cfg = wifi_pmf_config_t {
      capable: false,
      required: false,
    };

    let mut config = wifi_config_t {
      sta: wifi_sta_config_t {
        ssid: config.ssid,
        password: config.password,
        scan_method,
        bssid_set: config.bssid.is_some(),
        bssid,
        channel: config.channel.unwrap_or(0),
        listen_interval: config.listen_interval.unwrap_or(0),
        sort_method,
        threshold,
        pmf_cfg,
      },
    };

    set_mode(wifi_mode_t::WIFI_MODE_STA).unwrap();
    set_config(esp_interface_t::ESP_IF_WIFI_STA, &mut config).unwrap();

    Wifi { mode: PhantomData }
  }
}

#[cfg(target_device = "esp32")]
unsafe extern "C" fn wifi_scan_done_handler(
  event_handler_arg: *mut libc::c_void,
  event_base: esp_idf_sys::esp_event_base_t,
  event_id: i32,
  event_data: *mut libc::c_void,
) {
  let mut ap_num = 0;
  EspError::result(unsafe { esp_wifi_scan_get_ap_num(&mut ap_num) }).unwrap();

  hprintln!("Found {} APs:", ap_num);

  use core::mem::MaybeUninit;

  let mut aps: Vec<MaybeUninit<wifi_ap_record_t>> = vec![unsafe { MaybeUninit::uninit().assume_init() }; ap_num as usize];
  if EspError::result(unsafe { esp_wifi_scan_get_ap_records(&mut ap_num, aps.as_mut_ptr() as *mut wifi_ap_record_t) }).is_ok() {

    for i in 0..ap_num {
      let ap = unsafe { aps[i as usize].assume_init() };

      let ssid = ap.ssid.iter().take_while(|&c| *c != 0).map(|&c| unsafe { core::char::from_u32_unchecked(c as u32) }).collect::<String>();
      hprintln!("- {}: {}", i, ssid);
    }
  }
}

pub struct Sta;
pub struct Ap;

pub struct ApConfig {
  ssid: [u8; 32],
  password: [u8; 64],
  channel: u8,
  auth_mode: AuthMode,
  max_connection: u8,
  ssid_hidden: bool,
  beacon_interval: u16,
}

impl ApConfig {
  pub fn builder() -> ApConfigBuilder {
    ApConfigBuilder::default()
  }
}

pub struct ApConfigBuilder {
  ssid: Option<[u8; 32]>,
  password: [u8; 64],
  channel: u8,
  auth_mode: AuthMode,
  max_connection: u8,
  ssid_hidden: bool,
  beacon_interval: u16,
}

impl Default for ApConfigBuilder {
  fn default() -> Self {
    Self {
      ssid: Default::default(),
      password: [0; 64],
      channel: 0,
      auth_mode: AuthMode::Open,
      max_connection: 4,
      ssid_hidden: false,
      beacon_interval: 100,
    }
  }
}

impl ApConfigBuilder {
  pub fn ssid(&mut self, ssid: impl AsRef<str>) -> &mut Self {
    let mut ssid_buf: [u8; 32] = [0; 32];

    for (i, c) in ssid.as_ref().chars().take(31).enumerate()  {
      ssid_buf[i] = c as u8;
    }

    self.ssid = Some(ssid_buf);

    self
  }

  pub fn password(&mut self, password: impl AsRef<str>) -> &mut Self {
    let mut password_buf: [u8; 64] = [0; 64];

    for (i, c) in password.as_ref().chars().take(63).enumerate()  {
      password_buf[i] = c as u8;
    }

    self.password = password_buf;

    self
  }

  pub fn build(&self) -> ApConfig {
    ApConfig {
      ssid: self.ssid.unwrap(),
      password: self.password,
      channel: self.channel,
      auth_mode: self.auth_mode,
      max_connection: self.max_connection,
      ssid_hidden: self.ssid_hidden,
      beacon_interval: self.beacon_interval,
    }
  }
}

pub struct StaConfig {
  ssid: [u8; 32],
  password: [u8; 64],
  scan_method: ScanMethod,
  bssid: Option<[u8; 6]>,
  channel: Option<u8>,
  listen_interval: Option<u16>,
  sort_method: SortMethod,
  threshold: Option<ScanThreshold>,
}

#[derive(Clone, Copy)]
pub struct ScanThreshold {
  rssi: i8,
  auth_mode: AuthMode,
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub enum ScanMethod {
  Fast,
  Full,
}

impl Default for ScanMethod {
  fn default() -> Self {
    Self::Fast
  }
}

#[derive(Clone, Copy)]
pub enum SortMethod {
  BySignal,
}

impl Default for SortMethod {
  fn default() -> Self {
    Self::BySignal
  }
}

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

impl StaConfig {
  pub fn builder() -> StaConfigBuilder {
    StaConfigBuilder::default()
  }
}

pub struct StaConfigBuilder {
  ssid: Option<[u8; 32]>,
  password: [u8; 64],
  scan_method: ScanMethod,
  bssid: Option<[u8; 6]>,
  channel: Option<u8>,
  listen_interval: Option<u16>,
  sort_method: SortMethod,
  threshold: Option<ScanThreshold>,
}

impl Default for StaConfigBuilder {
  fn default() -> Self {
    Self {
      ssid: Default::default(),
      password: [0; 64],
      scan_method: Default::default(),
      bssid: Default::default(),
      channel: Default::default(),
      listen_interval: Default::default(),
      sort_method: Default::default(),
      threshold: Default::default(),
    }
  }
}

impl StaConfigBuilder {
  pub fn ssid(&mut self, ssid: impl AsRef<str>) -> &mut Self {
    let mut ssid_buf: [u8; 32] = [0; 32];

    for (i, c) in ssid.as_ref().chars().take(31).enumerate()  {
      ssid_buf[i] = c as u8;
    }

    self.ssid = Some(ssid_buf);

    self
  }

  pub fn password(&mut self, password: impl AsRef<str>) -> &mut Self {
    let mut password_buf: [u8; 64] = [0; 64];

    for (i, c) in password.as_ref().chars().take(63).enumerate()  {
      password_buf[i] = c as u8;
    }

    self.password = password_buf;

    self
  }

  pub fn build(&self) -> StaConfig {
    StaConfig {
      ssid: self.ssid.unwrap(),
      password: self.password,
      scan_method: self.scan_method,
      bssid: self.bssid,
      channel: self.channel,
      listen_interval: self.listen_interval,
      sort_method: self.sort_method,
      threshold: self.threshold,
    }
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

impl WifiError {
  pub fn wifi(self) -> Wifi<()> {
    Wifi { mode: PhantomData }
  }
}

#[must_use = "futures do nothing unless polled"]
pub struct ConnectFuture {
  state: ConnectFutureState,
}

use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Poll, Context, Waker};
use core::pin::Pin;

#[derive(Clone, Debug)]
pub enum WifiError {
  Internal(EspError),
  ConnectionError(wifi_err_reason_t),
}

impl From<EspError> for WifiError {
  fn from(esp_error: EspError) -> Self {
    Self::Internal(esp_error)
  }
}

impl core::fmt::Display for WifiError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Internal(esp_error) => esp_error.fmt(f),
      Self::ConnectionError(reason) => core::fmt::Debug::fmt(reason, f),
    }
  }
}

impl core::future::Future for ConnectFuture {
  type Output = Result<StaRunning, WifiError>;

  #[cfg(target_device = "esp8266")]
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    Poll::Pending
  }

  #[cfg(target_device = "esp32")]
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    use esp_idf_sys::{esp_event_handler_register, esp_event_handler_unregister, ip_event_t, wifi_event_t, IP_EVENT, WIFI_EVENT};

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
  event_base: esp_idf_sys::esp_event_base_t,
  event_id: i32,
  event_data: *mut libc::c_void,
) {
  use esp_idf_sys::{ip_event_t, ip_event_got_ip_t, wifi_event_sta_connected_t, wifi_event_sta_disconnected_t, wifi_event_t, IP_EVENT, WIFI_EVENT};

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

        let err = EspError::result(data.reason as esp_err_t).unwrap_err();
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
        let mut event: ip_event_got_ip_t = *(event_data as *mut ip_event_got_ip_t);
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
