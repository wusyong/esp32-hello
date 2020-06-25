use std::ffi::CStr;

use esp_idf_bindgen::{
  esp_err_t,
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
  ESP_ERR_NVS_NOT_FOUND,
};

use super::*;

/// Trait for retrieving data from non-volatile storage.
pub trait NvsGet: Sized {
  fn nvs_get(namespace: &NameSpace, key: &CStr) -> Result<Self, EspError>;
}

/// Trait for saving data in non-volatile storage.
pub trait NvsSet {
  fn nvs_set(&self, namespace: &mut NameSpace, key: &CStr) -> Result<(), EspError>;
}

impl<T> NvsSet for &T where T: NvsSet {
  fn nvs_set(&self, namespace: &mut NameSpace, key: &CStr) -> Result<(), EspError> {
    (*self).nvs_set(namespace, key)
  }
}

macro_rules! nvs_int {
  ($ty:ty as $as_ty:ty, $set_function:ident, $get_function:ident) => {
    impl NvsSet for $ty {
      fn nvs_set(&self, namespace: &mut NameSpace, key: &CStr) -> Result<(), EspError> {
        esp_ok!($set_function(namespace.handle, key.as_ptr(), *self as $as_ty))
      }
    }

    impl NvsGet for $ty {
      fn nvs_get(namespace: &NameSpace, key: &CStr) -> Result<Self, EspError> {
        let mut out_value = <$ty>::default();
        esp_ok!($get_function(namespace.handle, key.as_ptr(), &mut out_value as *mut $ty as *mut $as_ty))?;
        Ok(out_value)
      }
    }
  };
  ($ty:ty, $set_function:ident, $get_function:ident) => {
    nvs_int!($ty as $ty, $set_function, $get_function);
  };
}

nvs_int!(bool as u8, nvs_set_u8, nvs_get_u8);

nvs_int!( i8,  nvs_set_i8,  nvs_get_i8);
nvs_int!(i16, nvs_set_i16, nvs_get_i16);
nvs_int!(i32, nvs_set_i32, nvs_get_i32);
nvs_int!(i64, nvs_set_i64, nvs_get_i64);
nvs_int!( u8,  nvs_set_u8,  nvs_get_u8);
nvs_int!(u16, nvs_set_u16, nvs_get_u16);
nvs_int!(u32, nvs_set_u32, nvs_get_u32);
nvs_int!(u64, nvs_set_u64, nvs_get_u64);

impl NvsSet for &CStr {
  fn nvs_set(&self, namespace: &mut NameSpace, key: &CStr) -> Result<(), EspError> {
    esp_ok!(nvs_set_str(namespace.handle, key.as_ptr(), self.as_ptr()))
  }
}

impl NvsSet for CString {
  fn nvs_set(&self, namespace: &mut NameSpace, key: &CStr) -> Result<(), EspError> {
    self.as_c_str().nvs_set(namespace, key)
  }
}

impl NvsGet for CString {
  fn nvs_get(namespace: &NameSpace, key: &CStr) -> Result<Self, EspError> {
    let mut len = 0;
    esp_ok!(nvs_get_str(namespace.handle, key.as_ptr(), ptr::null_mut(), &mut len))?;

    let mut buffer = vec![0u8; len as usize];
    esp_ok!(nvs_get_str(namespace.handle, key.as_ptr(), buffer.as_mut_ptr() as *mut _, &mut len))?;

    Ok(unsafe { CString::from_vec_unchecked(buffer) })
  }
}

impl NvsSet for &[u8] {
  fn nvs_set(&self, namespace: &mut NameSpace, key: &CStr) -> Result<(), EspError> {
    esp_ok!(nvs_set_blob(namespace.handle, key.as_ptr(), self.as_ptr() as *const _, self.len() as u32))
  }
}

impl NvsSet for Vec<u8> {
  fn nvs_set(&self, namespace: &mut NameSpace, key: &CStr) -> Result<(), EspError> {
    self.as_slice().nvs_set(namespace, key)
  }
}

impl NvsGet for Vec<u8> {
  fn nvs_get(namespace: &NameSpace, key: &CStr) -> Result<Self, EspError> {
    let mut len = 0;
    esp_ok!(nvs_get_blob(namespace.handle, key.as_ptr(), ptr::null_mut(), &mut len))?;

    let mut buffer = vec![0u8; len as usize];
    esp_ok!(nvs_get_blob(namespace.handle, key.as_ptr(), buffer.as_mut_ptr() as *mut _, &mut len))?;
    Ok(buffer)
  }
}

impl NvsSet for &str {
  fn nvs_set(&self, namespace: &mut NameSpace, key: &CStr) -> Result<(), EspError> {
    self.as_bytes().nvs_set(namespace, key)
  }
}

impl NvsSet for String {
  fn nvs_set(&self, namespace: &mut NameSpace, key: &CStr) -> Result<(), EspError> {
    self.as_str().nvs_set(namespace, key)
  }
}


impl NvsGet for String {
  fn nvs_get(namespace: &NameSpace, key: &CStr) -> Result<Self, EspError> {
    let buffer = Vec::<u8>::nvs_get(namespace, key)?;
    String::from_utf8(buffer).map_err(|_| EspError { code: ESP_ERR_NVS_NOT_FOUND as esp_err_t })
  }
}
