use core::ptr;
use core::mem::MaybeUninit;

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

use crate::EspError;

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
    let key = CString::new(key).map_err(|_| EspError::from(ESP_ERR_NVS_NOT_FOUND as esp_err_t))?;
    T::nvs_get(self, key.as_ref())
  }

  pub fn set<T: NvsSet>(&mut self, key: &str, value: T) -> Result<(), EspError> {
    let key = CString::new(key).map_err(|_| EspError::from(ESP_ERR_NVS_NOT_FOUND as esp_err_t))?;
    value.nvs_set(self, key.as_ref())
  }
}

impl Drop for NameSpace {
  fn drop(&mut self) {
    unsafe { nvs_close(self.handle) };
  }
}

impl NonVolatileStorage {
  pub fn open(&mut self, name: &str) -> Result<NameSpace, EspError> {
    let name = CString::new(name).map_err(|_| EspError::from(ESP_ERR_NVS_NOT_FOUND as esp_err_t))?;

    let mut handle = MaybeUninit::<nvs_handle_t>::uninit();

    EspError::result(unsafe { nvs_open_from_partition(
      self.partition_name.as_ptr(),
      name.as_ptr(),
      nvs_open_mode_t::NVS_READWRITE,
      handle.as_mut_ptr(),
    ) })?;

    Ok(NameSpace { handle: unsafe { handle.assume_init() } })
  }

  fn init(&mut self) -> Result<(), EspError> {
    match EspError::result(unsafe { nvs_flash_init_partition(self.partition_name.as_ptr()) }) {
      Err(err) if err.code == ESP_ERR_NVS_NO_FREE_PAGES as esp_err_t || err.code == ESP_ERR_NVS_NEW_VERSION_FOUND as esp_err_t => {
        EspError::result(unsafe { nvs_flash_erase_partition(self.partition_name.as_ptr()) })?;
        EspError::result(unsafe { nvs_flash_init_partition(self.partition_name.as_ptr()) })
      }
      res => res,
    }
  }
}

impl Default for NonVolatileStorage {
  fn default() -> Self {
    let mut nvs = Self { partition_name: unsafe { CString::from_vec_unchecked(NVS_DEFAULT_PART_NAME.to_vec()) } };
    nvs.init().expect("failed to initialize default NVS partition");
    nvs
  }
}

impl Drop for NonVolatileStorage {
  fn drop(&mut self) {
    unsafe { nvs_flash_deinit_partition(self.partition_name.as_ptr()) };
  }
}
