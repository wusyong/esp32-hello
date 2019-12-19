#![feature(allocator_api)]
#![no_std]

extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::marker::PhantomData;

/// # Usage
///
/// ```
/// #![feature(alloc_error_handler)]
///
/// use core::alloc::Layout;
///
/// use alloc_esp32::Esp32Heap;
///
/// #[global_allocator]
/// static HEAP: Esp32Heap = Esp32Heap::system();
///
///
/// #[alloc_error_handler]
/// fn on_oom(_layout: Layout) -> ! {
///   loop {}
/// }
/// ```
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
