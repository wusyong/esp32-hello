#![no_main]

use std::sync::{Mutex, RwLock};
use std::time::Duration;
use std::thread::sleep;

#[no_mangle]
fn app_main() {
  let mutex = Mutex::new(0usize);
  assert_eq!(*mutex.lock().unwrap(), 0);
  *mutex.lock().unwrap() = 1;
  assert_eq!(*mutex.lock().unwrap(), 1);
  println!("Mutex: Success!");

  let rwlock = RwLock::new(0usize);
  assert_eq!(*rwlock.read().unwrap(), 0);
  *rwlock.write().unwrap() = 1;
  assert_eq!(*rwlock.read().unwrap(), 1);
  println!("RwLock: Success!");

  loop {
    sleep(Duration::from_secs(1));
  }
}
