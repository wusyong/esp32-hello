use core::ptr;

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;

use esp_idf_bindgen::{
  libc,
  esp_err_t,
  ESP_ERR_NVS_NO_FREE_PAGES,
  ESP_ERR_NVS_NEW_VERSION_FOUND,
  nvs_open_mode_t,
  nvs_handle_t,
  nvs_get_i8,
  nvs_get_u8,
  nvs_get_i16,
  nvs_get_u16,
  nvs_get_i32,
  nvs_set_i32,
  nvs_get_u32,
  nvs_get_i64,
  nvs_get_u64,
  nvs_get_blob,
  nvs_get_str,
  nvs_set_str,
  nvs_flash_init_partition,
  nvs_flash_erase_partition,
  nvs_flash_deinit_partition,
  nvs_open_from_partition,
  nvs_close,
  NVS_DEFAULT_PART_NAME,
};

use crate::{cstring, EspError};

#[derive(Debug)]
pub struct NonVolatileStorage {
  partition_name: [u8; 32],
}

#[derive(Debug)]
pub struct NameSpace {
  handle: nvs_handle_t,
}

pub trait GetValue: Sized {
  fn get(namespace: &NameSpace, key: &[libc::c_char]) -> Result<Self, EspError>;
}

pub trait SetValue {
  type Input;

  fn set(namespace: &NameSpace, key: &[libc::c_char], value: Self::Input) -> Result<(), EspError>;
}

impl GetValue for i8 {
  fn get(namespace: &NameSpace, key: &[libc::c_char]) -> Result<Self, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_i8(namespace.handle, key.as_ptr() as *const _, &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for i16 {
  fn get(namespace: &NameSpace, key: &[libc::c_char]) -> Result<Self, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_i16(namespace.handle, key.as_ptr() as *const _, &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for i32 {
  fn get(namespace: &NameSpace, key: &[libc::c_char]) -> Result<Self, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_i32(namespace.handle, key.as_ptr() as *const _, &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for i64 {
  fn get(namespace: &NameSpace, key: &[libc::c_char]) -> Result<Self, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_i64(namespace.handle, key.as_ptr() as *const _, &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for u8 {
  fn get(namespace: &NameSpace, key: &[libc::c_char]) -> Result<Self, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_u8(namespace.handle, key.as_ptr() as *const _, &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for u16 {
  fn get(namespace: &NameSpace, key: &[libc::c_char]) -> Result<Self, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_u16(namespace.handle, key.as_ptr() as *const _, &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for u32 {
  fn get(namespace: &NameSpace, key: &[libc::c_char]) -> Result<Self, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_u32(namespace.handle, key.as_ptr() as *const _, &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for u64 {
  fn get(namespace: &NameSpace, key: &[libc::c_char]) -> Result<Self, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_u64(namespace.handle, key.as_ptr() as *const _, &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for String {
  fn get(namespace: &NameSpace, key: &[libc::c_char]) -> Result<Self, EspError> {
    let mut len = 0;
    EspError::result(unsafe { nvs_get_str(namespace.handle, key.as_ptr() as *const _, ptr::null_mut(), &mut len) })?;

    let mut buffer = vec![0; len as usize];
    EspError::result(unsafe { nvs_get_str(namespace.handle, key.as_ptr() as *const _, buffer.as_mut_ptr(), &mut len) })?;

    Ok(buffer.into_iter().take(len as usize - 1).map(|c| c as u8 as char).collect())
  }
}

impl SetValue for i32 {
  type Input = i32;

  fn set(namespace: &NameSpace, key: &[libc::c_char], value: Self::Input) -> Result<(), EspError> {
    EspError::result(unsafe { nvs_set_i32(namespace.handle, key.as_ptr() as *const _, value) })
  }
}

impl<'a> SetValue for &'a str {
  type Input = &'a str;

  fn set(namespace: &NameSpace, key: &[libc::c_char], value: Self::Input) -> Result<(), EspError> {
    let value = cstring!(value);
    EspError::result(unsafe { nvs_set_str(namespace.handle, key.as_ptr() as *const _, value.as_ptr()) })
  }
}

impl NameSpace {
  pub fn get<T: GetValue + Sized>(&self, key: &str) -> Result<T, EspError> {
    let key = cstring!(key);
    T::get(self, &key)
  }

  pub fn set<T: SetValue>(&mut self, key: &str, value: T::Input) -> Result<(), EspError> {
    let key = cstring!(key);
    T::set(self, &key, value)
  }
}

impl Drop for NameSpace {
  fn drop(&mut self) {
    unsafe { nvs_close(self.handle) };
  }
}

impl NonVolatileStorage {
  pub fn open(&mut self, name: &str) -> Result<NameSpace, EspError> {
    let mut handle: nvs_handle_t = Default::default();
    let name = cstring!(name);
    EspError::result(unsafe { nvs_open_from_partition(
      &self.partition_name as *const _ as *const libc::c_char,
      name.as_ptr() as *const _ as *const libc::c_char,
      nvs_open_mode_t::NVS_READWRITE,
      &mut handle,
    ) })?;

    Ok(NameSpace { handle })
  }

  pub fn default() -> Result<Self, EspError> {
    let mut nvs = Self { partition_name: Default::default() };
    nvs.partition_name[..4].copy_from_slice(NVS_DEFAULT_PART_NAME);
    Ok(nvs)
  }
}

impl Drop for NonVolatileStorage {
  fn drop(&mut self) {
    unsafe { nvs_flash_deinit_partition(self.partition_name.as_ptr() as *const _) };
  }
}

impl NonVolatileStorage {
  pub fn flash_init(&mut self) -> Result<(), EspError> {
    let part_name = self.partition_name.as_ptr() as *const _;

    match EspError::result(unsafe { nvs_flash_init_partition(part_name) }) {
      Ok(()) => Ok(()),
      Err(err) if err.code == ESP_ERR_NVS_NO_FREE_PAGES as esp_err_t || err.code == ESP_ERR_NVS_NEW_VERSION_FOUND as esp_err_t => {
        EspError::result(unsafe { nvs_flash_erase_partition(part_name) })?;
        EspError::result(unsafe { nvs_flash_init_partition(part_name) })
      }
      Err(err) => Err(err),
    }
  }
}
