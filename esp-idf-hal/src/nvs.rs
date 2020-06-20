use core::ptr;
use core::mem::MaybeUninit;

extern crate alloc;

use alloc::string::String;

use std::ffi::{CStr, CString};

use esp_idf_bindgen::{
  esp_err_t,
  ESP_ERR_NVS_NO_FREE_PAGES,
  ESP_ERR_NVS_NEW_VERSION_FOUND,
  nvs_open_mode_t,
  nvs_handle_t,
  nvs_get_i8,
  nvs_set_i8,
  nvs_get_u8,
  nvs_set_u8,
  nvs_get_i16,
  nvs_set_i16,
  nvs_get_u16,
  nvs_set_u16,
  nvs_get_i32,
  nvs_set_i32,
  nvs_get_u32,
  nvs_set_u32,
  nvs_get_i64,
  nvs_set_i64,
  nvs_get_u64,
  nvs_set_u64,
  nvs_get_blob,
  nvs_set_blob,
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

pub trait GetValue {
  type Output;

  fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError>;
}

pub trait SetValue {
  fn set(namespace: &mut NameSpace, key: &CStr, value: Self) -> Result<(), EspError>;
}

macro_rules! nvs_int {
  ($ty:ty, $set_function:ident, $get_function:ident) => {
    impl SetValue for $ty {
      fn set(namespace: &mut NameSpace, key: &CStr, value: Self) -> Result<(), EspError> {
        EspError::result(unsafe { $set_function(namespace.handle, key.as_ptr(), value) })
      }
    }

    impl GetValue for $ty {
      type Output = $ty;

      fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError> {
        let mut out_value: Self::Output = 0;
        EspError::result(unsafe { $get_function(namespace.handle, key.as_ptr(), &mut out_value) })?;
        Ok(out_value)
      }
    }
  }
}

nvs_int!( i8,  nvs_set_i8,  nvs_get_i8);
nvs_int!(i16, nvs_set_i16, nvs_get_i16);
nvs_int!(i32, nvs_set_i32, nvs_get_i32);
nvs_int!(i64, nvs_set_i64, nvs_get_i64);
nvs_int!( u8,  nvs_set_u8,  nvs_get_u8);
nvs_int!(u16, nvs_set_u16, nvs_get_u16);
nvs_int!(u32, nvs_set_u32, nvs_get_u32);
nvs_int!(u64, nvs_set_u64, nvs_get_u64);

impl<'a> SetValue for &'a CStr {
  fn set(namespace: &mut NameSpace, key: &CStr, value: Self) -> Result<(), EspError> {
    EspError::result(unsafe { nvs_set_str(namespace.handle, key.as_ptr(), value.as_ptr()) })
  }
}

impl GetValue for CString {
  type Output = CString;

  fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError> {
    let mut len = 0;
    EspError::result(unsafe { nvs_get_str(namespace.handle, key.as_ptr(), ptr::null_mut(), &mut len) })?;

    let mut buffer = vec![0u8; len as usize];
    EspError::result(unsafe { nvs_get_str(namespace.handle, key.as_ptr(), buffer.as_mut_ptr() as *mut _, &mut len) })?;

    Ok(unsafe { CString::from_vec_unchecked(buffer) })
  }
}

impl<'a> SetValue for &'a [u8] {
  fn set(namespace: &mut NameSpace, key: &CStr, value: Self) -> Result<(), EspError> {
    EspError::result(unsafe { nvs_set_blob(namespace.handle, key.as_ptr(), value.as_ptr() as *const _, value.len() as u32) })
  }
}

impl GetValue for Vec<u8> {
  type Output = Vec<u8>;

  fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError> {
    let mut len = 0;
    EspError::result(unsafe { nvs_get_blob(namespace.handle, key.as_ptr(), ptr::null_mut(), &mut len) })?;

    let mut buffer = vec![0u8; len as usize];
    EspError::result(unsafe { nvs_get_blob(namespace.handle, key.as_ptr(), buffer.as_mut_ptr() as *mut _, &mut len) })?;
    Ok(buffer)
  }
}

impl GetValue for String {
  type Output = String;

  fn get(namespace: &NameSpace, key: &CStr) -> Result<Self::Output, EspError> {
    let buffer = Vec::<u8>::get(namespace, key)?;
    String::from_utf8(buffer).map_err(|_| EspError::from(ESP_ERR_NVS_NOT_FOUND as esp_err_t))
  }
}

impl<'a> SetValue for &'a str {
  fn set(namespace: &mut NameSpace, key: &CStr, value: Self) -> Result<(), EspError> {
    <&[u8]>::set(namespace, key, value.as_bytes())
  }
}

impl SetValue for String {
  fn set(namespace: &mut NameSpace, key: &CStr, value: Self) -> Result<(), EspError> {
    <&str>::set(namespace, key, &value)
  }
}

impl NameSpace {
  pub fn get<T: GetValue>(&self, key: &str) -> Result<T::Output, EspError> {
    let key = CString::new(key).map_err(|_| EspError::from(ESP_ERR_NVS_NOT_FOUND as esp_err_t))?;
    T::get(self, key.as_ref())
  }

  pub fn set<T: SetValue>(&mut self, key: &str, value: T) -> Result<(), EspError> {
    let key = CString::new(key).map_err(|_| EspError::from(ESP_ERR_NVS_NOT_FOUND as esp_err_t))?;
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
