#![feature(allocator_api)]
#![no_std]

extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::marker::PhantomData;

pub struct Esp32Heap<T = ()> { heap: PhantomData<T> }

impl Esp32Heap {
  pub const fn system() -> Self {
    Self { heap: PhantomData }
  }
}

unsafe impl<T: Sync> Sync for Esp32Heap<T> {}

extern "C" {
  fn malloc(_: usize) -> *mut u8;
  fn calloc(_: usize, _: usize) -> *mut u8;
  fn free(_: *mut u8);
}

unsafe impl GlobalAlloc for Esp32Heap {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    malloc(layout.size())
  }

  unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
    calloc(1, layout.size())
  }

  unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
    free(ptr);
  }
}
