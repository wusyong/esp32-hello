#![feature(never_type)]
#![no_main]

extern crate alloc;

use alloc::string::ToString;
use alloc::string::String;

#[macro_use]
extern crate std;

use std::io::{Read, Write};
use std::thread::{self, sleep};
use std::time::Duration;
use std::sync::{Mutex, RwLock};
use std::net::Ipv4Addr;
use std::net::TcpListener;

use embedded_hal::digital::v2::OutputPin;

use esp32_hal::{*, gpio::*, nvs::*, wifi::*};

use futures::executor::block_on;

use core::ptr;
use esp_idf_sys::*;

mod wifi_manager;
use wifi_manager::*;

mod wifi_credentials;
use wifi_credentials::*;

#[no_mangle]
pub fn app_main() {
  block_on(async {
    if let Err(err) = rust_blink_and_write().await {
      println!("{}", err);
    }
  })
}

async fn rust_blink_and_write() -> Result<!, EspError> {
  let mut gpio = GPIO22::into_input_output();

    let mut nvs = NonVolatileStorage::default()?;

    let wifi = Wifi::init(&mut nvs);

    println!("AP started.");

    // esp32_hal::wifi::wifi_scan(true, false, 1000)?;

    let mutex = Mutex::new(0usize);
    println!("mutex value = {:?}", *mutex.lock().unwrap());
    *mutex.lock().unwrap() = 1;
    println!("mutex value = {:?}", *mutex.lock().unwrap());

    let rwlock = RwLock::new(0usize);
    println!("rwlock value = {:?}", *rwlock.read().unwrap());
    *rwlock.write().unwrap() = 1;
    println!("rwlock value = {:?}", *rwlock.read().unwrap());

    let mut namespace = nvs.open("wifi").map_err(|e| e.to_string());
    println!("namespace: {:?}", namespace);

    let t = thread::Builder::new()
      .name("hello_thread".into())
      .stack_size(8192)
      .spawn(|| {
        println!("HELLO, WORLD!");
        42
      });

    println!("Thread spawn result: {:?}", t);
    println!("Thread join result: {:?}", t.map(|t| t.join().unwrap()));

    thread::Builder::new()
      .name("blink_thread".into())
      .spawn(move || {
        loop {
          gpio.set_low().unwrap();
          sleep(Duration::from_millis(100));
          gpio.set_high().unwrap();
          sleep(Duration::from_secs(3));
        }
      })
      .unwrap();

    thread::Builder::new()
      .name("server_thread".into())
      .stack_size(8192)
      .spawn(move || block_on(async {
        let mac = mac_address(MacAddressType::Ap);
        let ssid = format!("ESP {:#}", mac);
        println!("Starting Access Point '{}' …", ssid);

        let ap_config = ApConfig::builder()
          .ssid(&ssid)
          .build();

        let ap = wifi.into_ap(&ap_config);

        let mut ap_running = Some(ap.start());
        let mut sta_running = None;

        let stream = TcpListener::bind("0.0.0.0:80").expect("failed starting TCP listener");

        loop {
          match stream.accept() {
            Ok((client, addr)) => {
              match handle_request(client, addr, &ap_config, ap_running.take(), sta_running.take()).await {
                Ok((ap, sta)) => {
                  ap_running = ap;
                  sta_running = sta;
                },
                Err(err) => {
                  eprintln!("Failed to handle request: {:?}", err);
                },
              }
           },
            Err(e) => {
              if e.kind() != std::io::ErrorKind::WouldBlock {
                eprintln!("couldn't get client: {:?}", e);
              }
            },
          }

          thread::yield_now();
        }
      }))
      .unwrap();

    loop {
      if let Ok(ref mut namespace) = namespace {
        println!("GOT VALUE: {:?}", namespace.get::<i32>("forty-two").map_err(|e| e.to_string()));
        println!("GOT VALUE: {:?}", namespace.get::<String>("hakuna").map_err(|e| e.to_string()));

        println!("Setting value: {:?}", namespace.set::<i32>("forty-two", 42).map_err(|e| e.to_string()));
        println!("Setting value: {:?}", namespace.set::<&str>("hakuna", "matata").map_err(|e| e.to_string()));
      }

      sleep(Duration::from_secs(5))
    }
  }
