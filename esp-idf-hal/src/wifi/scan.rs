use core::cmp;
use core::future::Future;
use core::mem::{self, MaybeUninit};
use core::pin::Pin;
use core::ptr;
use core::task::{Poll, Context, Waker};
use std::time::Duration;

use esp_idf_bindgen::{
  libc,
  esp_event_base_t,
  esp_event_handler_register,
  esp_event_handler_unregister,
  esp_wifi_scan_start,
  esp_wifi_scan_get_ap_num,
  esp_wifi_scan_get_ap_records,
  esp_wifi_set_mode,
  esp_wifi_start,
  esp_wifi_stop,
  wifi_ap_record_t,
  wifi_event_t,
  WIFI_EVENT,
  wifi_mode_t,
  wifi_scan_config_t,
  wifi_scan_time_t,
  wifi_active_scan_time_t,
  wifi_scan_type_t,
};
use macaddr::MacAddr6;

use crate::EspError;
use super::{Ssid, WifiError};

#[derive(Debug, Clone)]
pub enum ScanType {
  Active { min: Duration, max: Duration },
  Passive { max: Duration },
}

impl Default for ScanType {
  fn default() -> Self {
    Self::Active { min: Duration::from_millis(0), max: Duration::from_millis(0) }
  }
}

#[derive(Default, Debug, Clone)]
pub struct ScanConfig {
  ssid: Option<Ssid>,
  bssid: Option<MacAddr6>,
  channel: u8,
  show_hidden: bool,
  scan_type: ScanType,
}

impl ScanConfig {
  pub fn builder() -> ScanConfigBuilder {
    ScanConfigBuilder {
      ssid: None,
      bssid: None,
      channel: 0,
      show_hidden: false,
      scan_type: Default::default(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ScanConfigBuilder {
  ssid: Option<Ssid>,
  bssid: Option<MacAddr6>,
  channel: u8,
  show_hidden: bool,
  scan_type: ScanType,
}

impl ScanConfigBuilder {
  pub fn ssid(mut self, ssid: impl Into<Option<Ssid>>) -> ScanConfigBuilder {
    self.ssid = ssid.into();
    self
  }

  pub fn bssid(mut self, bssid: impl Into<Option<MacAddr6>>) -> ScanConfigBuilder {
    self.bssid = bssid.into();
    self
  }


  pub fn channel(mut self, channel: u8) -> ScanConfigBuilder {
    self.channel = channel;
    self
  }

  pub fn show_hidden(mut self, show_hidden: bool) -> ScanConfigBuilder {
    self.show_hidden = show_hidden;
    self
  }

  pub fn scan_type(mut self, scan_type: ScanType) -> ScanConfigBuilder {
    if let ScanType::Active { min, max } = scan_type {
      assert!(min <= max);
    }
    self.scan_type = scan_type;
    self
  }

  pub fn build(self) -> ScanConfig {
    let Self { ssid, bssid, channel, show_hidden, scan_type } = self;
    ScanConfig { ssid, bssid, channel, show_hidden, scan_type }
  }
}

#[derive(Debug, Clone)]
pub struct ApRecord {
  ssid: Ssid,
  bssid: MacAddr6,
}

impl ApRecord {
  pub fn ssid(&self) -> &Ssid {
    &self.ssid
  }

  pub fn bssid(&self) -> &MacAddr6 {
    &self.bssid
  }
}

#[derive(Debug)]
enum ScanFutureState {
  Starting(*const Waker),
  Failed(WifiError),
  Done,
}

#[must_use = "futures do nothing unless polled"]
#[derive(Debug)]
pub struct ScanFuture {
  state: Pin<Box<ScanFutureState>>,
}

impl ScanFuture {
  #[inline]
  pub(crate) fn new(config: &ScanConfig) -> Self {
    let mut state = Box::pin(ScanFutureState::Starting(ptr::null()));

    if let Err(err) = EspError::result(unsafe { esp_wifi_set_mode(wifi_mode_t::WIFI_MODE_STA) }) {
      return Self { state: Box::pin(ScanFutureState::Failed(err.into())) };
    }

    if let Err(err) = EspError::result(unsafe { esp_wifi_start() }) {
      return Self { state: Box::pin(ScanFutureState::Failed(err.into())) };
    }

    let (scan_type, scan_time) = match config.scan_type {
      ScanType::Active { min, max } => (
        wifi_scan_type_t::WIFI_SCAN_TYPE_ACTIVE,
        wifi_scan_time_t {
          active: wifi_active_scan_time_t {
            min: cmp::min(u32::max_value() as u128, min.as_millis()) as u32,
            max: cmp::min(u32::max_value() as u128, max.as_millis()) as u32,
          },
          passive: 0,
        },
      ),
      ScanType::Passive { max } => (
        wifi_scan_type_t::WIFI_SCAN_TYPE_PASSIVE,
        wifi_scan_time_t {
          active: wifi_active_scan_time_t { min: 0, max: 0 },
          passive: cmp::min(u32::max_value() as u128, max.as_millis()) as u32,
        },
      )
    };

    let config = wifi_scan_config_t {
      ssid: config.ssid.as_ref().map_or_else(ptr::null_mut, |ssid| ssid.ssid.as_ptr() as *mut _),
      bssid: config.bssid.as_ref().map_or_else(ptr::null_mut, |bssid| bssid as *const _ as *mut _),
      channel: config.channel,
      show_hidden: config.show_hidden,
      scan_type,
      scan_time,
    };

    if let Err(err) = register_scan_done_handler((&mut *state) as *mut _) {
      return Self { state: Box::pin(ScanFutureState::Failed(err.into())) };
    };

    if let Err(err) = EspError::result(unsafe { esp_wifi_scan_start(&config, false) }) {
      let _ = unregister_scan_done_handler();
      return Self { state: Box::pin(ScanFutureState::Failed(err.into())) };
    };

    Self { state }
  }
}

impl Future for ScanFuture {
  type Output = Result<Vec<ApRecord>, WifiError>;

  #[cfg(target_device = "esp8266")]
  fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    Poll::Pending
  }

  #[cfg(target_device = "esp32")]
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    match &mut *self.state {
      ScanFutureState::Starting(ref mut waker) => {
        *waker = cx.waker() as *const _;
        Poll::Pending
      },
      ScanFutureState::Failed(ref mut err) => {
        Poll::Ready(Err(mem::replace(err, unsafe { MaybeUninit::uninit().assume_init() })))
      },
      ScanFutureState::Done => {
        let unregister = unregister_scan_done_handler();
        let aps = get_ap_records();
        let stop = EspError::result(unsafe { esp_wifi_stop() });

        unregister?;
        let aps = aps?;
        stop?;

        Poll::Ready(Ok(aps))
      }
    }
  }
}

#[inline]
fn get_ap_records() -> Result<Vec<ApRecord>, EspError> {
  let mut ap_num = 0;
  EspError::result(unsafe { esp_wifi_scan_get_ap_num(&mut ap_num) })?;

  let mut aps: Vec<MaybeUninit<wifi_ap_record_t>> = vec![MaybeUninit::uninit(); ap_num as usize];
  EspError::result(unsafe { esp_wifi_scan_get_ap_records(&mut ap_num as _, aps.as_mut_ptr() as *mut wifi_ap_record_t) })?;

  Ok(aps.into_iter().map(|ap| {
    let ap = unsafe { ap.assume_init() };

    let ssid_len = memchr::memchr(0, &ap.ssid).unwrap_or(ap.ssid.len());
    let ssid = unsafe { Ssid::from_bytes_unchecked(&ap.ssid[..ssid_len]) };

    let bssid = MacAddr6::from(ap.bssid);

    ApRecord { ssid, bssid }
  }).collect())
}

#[inline]
fn register_scan_done_handler(b: *mut ScanFutureState) -> Result<(), EspError> {
  EspError::result(unsafe {
    esp_event_handler_register(WIFI_EVENT, wifi_event_t::WIFI_EVENT_SCAN_DONE as _, Some(wifi_scan_done_handler), b as *mut _)
  })
}

#[inline]
fn unregister_scan_done_handler() -> Result<(), EspError> {
  EspError::result(unsafe {
    esp_event_handler_unregister(WIFI_EVENT, wifi_event_t::WIFI_EVENT_SCAN_DONE as _, Some(wifi_scan_done_handler))
  })
}

#[cfg(target_device = "esp32")]
extern "C" fn wifi_scan_done_handler(
  event_handler_arg: *mut libc::c_void,
  _event_base: esp_event_base_t,
  _event_id: i32,
  _event_data: *mut libc::c_void,
) {
  let state =  unsafe { &mut *(event_handler_arg as *mut ScanFutureState) };
  if let ScanFutureState::Starting(waker) = mem::replace(state, ScanFutureState::Done) {
    if let Some(waker) = unsafe { waker.as_ref() } {
      waker.wake_by_ref();
    }
  }
}
