use core::ptr;
use core::mem::MaybeUninit;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::ffi::CString;

use esp_idf_bindgen::{
  esp_err_t,
  ESP_ERR_NVS_NO_FREE_PAGES,
  ESP_ERR_NVS_NEW_VERSION_FOUND,
  nvs_open_mode_t,
  nvs_handle_t,
  nvs_flash_init_partition,
  nvs_flash_erase_partition,
  nvs_flash_deinit_partition,
  nvs_open_from_partition,
  nvs_close,
  NVS_DEFAULT_PART_NAME,
  ESP_ERR_NVS_NOT_FOUND,
};

use super::*;

mod get_set;
pub use get_set::*;

#[derive(Debug)]
pub struct NonVolatileStorage {
  partition_name: CString,
}

#[derive(Debug)]
pub struct NameSpace {
  handle: nvs_handle_t,
}

impl NameSpace {
  pub fn get<T: NvsGet>(&self, key: &str) -> Result<T, EspError> {
    let key = CString::new(key).map_err(|_| EspError { code: ESP_ERR_NVS_NOT_FOUND as esp_err_t })?;
    T::nvs_get(self, key.as_ref())
  }

  pub fn set<T: NvsSet>(&mut self, key: &str, value: T) -> Result<(), EspError> {
    let key = CString::new(key).map_err(|_| EspError { code: ESP_ERR_NVS_NOT_FOUND as esp_err_t })?;
    value.nvs_set(self, key.as_ref())
  }
}

impl Drop for NameSpace {
  fn drop(&mut self) {
    unsafe { nvs_close(self.handle) };
  }
}

static DEFAULT_INSTANCES: AtomicUsize = AtomicUsize::new(0);

impl NonVolatileStorage {
  pub fn open(&mut self, name: &str) -> Result<NameSpace, EspError> {
    let name = CString::new(name).map_err(|_| EspError { code: ESP_ERR_NVS_NOT_FOUND as esp_err_t })?;

    let mut handle = MaybeUninit::<nvs_handle_t>::uninit();

    esp_ok!(nvs_open_from_partition(
      self.partition_name.as_ptr(),
      name.as_ptr(),
      nvs_open_mode_t::NVS_READWRITE,
      handle.as_mut_ptr(),
    ))?;

    Ok(NameSpace { handle: unsafe { handle.assume_init() } })
  }

  fn init(partition_name: &CStr) -> Result<(), EspError> {
    esp_ok!(nvs_flash_init_partition(partition_name.as_ptr()))
  }

  pub(crate) fn init_default() {
    loop {
      match DEFAULT_INSTANCES.compare_and_swap(0, 1, Ordering::SeqCst) {
        0 => {
          match esp_ok!(nvs_flash_init_partition(NVS_DEFAULT_PART_NAME.as_ptr() as *const _)) {
            Err(err) if err.code == ESP_ERR_NVS_NO_FREE_PAGES as esp_err_t || err.code == ESP_ERR_NVS_NEW_VERSION_FOUND as esp_err_t => {
              assert_esp_ok!(nvs_flash_erase_partition(NVS_DEFAULT_PART_NAME.as_ptr() as *const _));
              assert_esp_ok!(nvs_flash_init_partition(NVS_DEFAULT_PART_NAME.as_ptr() as *const _));
            }
            Err(err) => assert_esp_ok!(err.code),
            Ok(()) => (),
          }
          DEFAULT_INSTANCES.fetch_add(1, Ordering::SeqCst);
          return;
        },
        1 => continue,
        _ => {
          DEFAULT_INSTANCES.fetch_add(1, Ordering::SeqCst);
          return
        },
      }
    }
  }

  pub(crate) fn deinit_default() {
    loop {
      match DEFAULT_INSTANCES.compare_and_swap(2, 1, Ordering::SeqCst) {
        2 => {
          unsafe { nvs_flash_deinit_partition(NVS_DEFAULT_PART_NAME.as_ptr() as *const _) };
          DEFAULT_INSTANCES.fetch_sub(1, Ordering::SeqCst);
          return;
        },
        1 => continue,
        _ => {
          DEFAULT_INSTANCES.fetch_sub(1, Ordering::SeqCst);
          return
        },
      }
    }
  }
}

impl Default for NonVolatileStorage {
  fn default() -> Self {
    let mut default_partition = NVS_DEFAULT_PART_NAME.to_vec();
    default_partition.pop();
    let mut nvs = Self { partition_name: unsafe { CString::from_vec_unchecked(default_partition) } };
    Self::init_default();
    nvs
  }
}

impl Drop for NonVolatileStorage {
  fn drop(&mut self) {
    if self.partition_name.as_bytes_with_nul() == NVS_DEFAULT_PART_NAME {
      Self::deinit_default();
    } else {
      unsafe { nvs_flash_deinit_partition(self.partition_name.as_ptr()) };
    }
  }
}
