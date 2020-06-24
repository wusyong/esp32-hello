use std::marker::PhantomData;

use esp_idf_bindgen::*;

#[derive(Debug)]
pub struct Heap {
  _marker: PhantomData<()>,
}

impl Heap {
  pub fn total_size() -> usize {
    unsafe { heap_caps_get_total_size(MALLOC_CAP_DEFAULT) as usize }
  }

  pub fn free_size() -> usize {
    unsafe { heap_caps_get_free_size(MALLOC_CAP_DEFAULT) as usize }
  }
}
