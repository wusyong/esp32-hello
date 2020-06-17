use core::ptr;
use core::mem::MaybeUninit;

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;

use std::ffi::{CStr, CString};

use esp_idf_bindgen::{
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
  ESP_ERR_NVS_NOT_FOUND,
};

use crate::EspError;

#[derive(Debug)]
pub struct NonVolatileStorage {
  partition_name: CString,
}

#[derive(Debug)]
pub struct NameSpace {
  handle: nvs_handle_t,
}

pub trait GetValue: Sized {
  type Output;

  fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError>;
}

pub trait SetValue {
  type Input;

  fn set(namespace: &NameSpace, key: &CStr, value: Self::Input) -> Result<(), EspError>;
}

impl GetValue for i8 {
  type Output = i8;

  fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_i8(namespace.handle, key.as_ptr(), &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for i16 {
  type Output = i16;

  fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_i16(namespace.handle, key.as_ptr(), &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for i32 {
  type Output = i32;

  fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_i32(namespace.handle, key.as_ptr(), &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for i64 {
  type Output = i64;

  fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_i64(namespace.handle, key.as_ptr(), &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for u8 {
  type Output = u8;

  fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_u8(namespace.handle, key.as_ptr(), &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for u16 {
  type Output = u16;

  fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_u16(namespace.handle, key.as_ptr(), &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for u32 {
  type Output = u32;

  fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_u32(namespace.handle, key.as_ptr(), &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for u64 {
  type Output = u64;

  fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError> {
    let mut out_value: Self = 0;
    EspError::result(unsafe { nvs_get_u64(namespace.handle, key.as_ptr(), &mut out_value) })?;
    Ok(out_value)
  }
}

impl GetValue for String {
  type Output = String;

  fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError> {
    let mut len = 0;
    EspError::result(unsafe { nvs_get_str(namespace.handle, key.as_ptr(), ptr::null_mut(), &mut len) })?;

    let mut buffer = vec![0u8; len as usize];
    EspError::result(unsafe { nvs_get_str(namespace.handle, key.as_ptr(), buffer.as_mut_ptr() as *mut _, &mut len) })?;
    buffer.truncate(len as usize - 1);

    Ok(String::from_utf8(buffer).map_err(|_| EspError::from(ESP_ERR_NVS_NOT_FOUND as esp_err_t))?)
  }
}

impl SetValue for i32 {
  type Input = i32;

  fn set(namespace: &NameSpace, key: &CStr, value: Self::Input) -> Result<(), EspError> {
    EspError::result(unsafe { nvs_set_i32(namespace.handle, key.as_ptr(), value) })
  }
}

impl<'a> SetValue for &'a str {
  type Input = &'a str;

  fn set(namespace: &NameSpace, key: &CStr, value: Self::Input) -> Result<(), EspError> {
    let value = CString::new(value).unwrap();
    EspError::result(unsafe { nvs_set_str(namespace.handle, key.as_ptr(), value.as_ptr()) })
  }
}

impl NameSpace {
  pub fn get<T: GetValue>(&self, key: &str) -> Result<T::Output, EspError> {
    let key = CString::new(key).unwrap();
    T::get(self, key.as_ref())
  }

  pub fn set<T: SetValue>(&mut self, key: &str, value: T::Input) -> Result<(), EspError> {
    let key = CString::new(key).unwrap();
    T::set(self, key.as_ref(), value)
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
