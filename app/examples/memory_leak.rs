#![no_main]

use std::{time::Duration, thread};

use esp_idf_hal::Heap;

#[no_mangle]
fn app_main() {
  eprintln!("Total Memory: {}", Heap::total_size());

  let size_before = Heap::free_size();
  eprintln!("Free Memory (Main Thread): {}", size_before);

  thread::Builder::new().stack_size(8192).spawn(|| {
    eprintln!("Free Memory (Thread 1): {}", Heap::free_size());

    thread::Builder::new().stack_size(8192).spawn(|| {
      eprintln!("Free Memory (Thread 2): {}", Heap::free_size());
    }).unwrap();
  }).unwrap();

  thread::sleep(Duration::from_secs(1));

  let size_after = Heap::free_size();
  eprintln!("Free Memory (Main Thread): {}", size_after);

  let leaked_memory = if size_after < size_before { size_before - size_after } else { 0 };
  eprintln!("Leaked Memory: {}", leaked_memory);
}
