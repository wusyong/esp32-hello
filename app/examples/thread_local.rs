#![no_main]

use std::cell::RefCell;
use std::thread::{self, sleep};
use std::time::Duration;

thread_local! {
  pub static FOO: RefCell<u32> = RefCell::new(0);
}

#[no_mangle]
fn app_main() {
  FOO.with(|f| {
    *f.borrow_mut() += 1;
  });

  thread::spawn(|| {
    FOO.with(|f| {
      *f.borrow_mut() += 1;
    });

    FOO.with(|f| {
      assert_eq!(1, *f.borrow());
      println!("Thread 1: Success!");
    });
  });

  thread::spawn(|| {
    FOO.with(|f| {
      *f.borrow_mut() += 1;
    });

    FOO.with(|f| {
      assert_eq!(1, *f.borrow());
      println!("Thread 2: Success!");
    });
  });

  FOO.with(|f| {
    assert_eq!(1, *f.borrow());
    println!("Main Thread: Success!");
  });


  loop {
    sleep(Duration::from_secs(1));
  }
}
