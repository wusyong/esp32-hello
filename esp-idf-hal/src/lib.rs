#![feature(never_type)]
#![feature(const_cstr_unchecked)]
#![warn(missing_debug_implementations)]

use std::ffi::CStr;
use std::str;

#[macro_use]
extern crate alloc;

#[macro_use]
mod esp_error;
pub use esp_error::EspError;

pub mod ets;
pub mod interface;
mod heap;
pub use heap::Heap;
pub mod wifi;
pub mod nvs;
