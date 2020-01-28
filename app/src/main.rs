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
use std::net::{Ipv4Addr, SocketAddrV4};
use std::net::TcpListener;

use embedded_hal::digital::v2::OutputPin;

use esp32_hal::{*, gpio::*, nvs::*, wifi::*};

use futures::executor::block_on;

use core::ptr;
use esp_idf_sys::*;

mod wifi_manager;
use wifi_manager::*;

mod dns;

#[no_mangle]
pub fn app_main() {
  // dbg!(std::net::ToSocketAddrs::to_socket_addrs("0.0.0.0:80")).unwrap();

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

    let mut namespace = nvs.open("wifi")?;
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
      .name("dns_thread".into())
      .stack_size(8192)
      .spawn(dns::server)
      .unwrap();

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
        let ap_ssid = format!("ESP {}", mac);

        let ap_config = ApConfig::builder()
          .ssid(&ap_ssid)
          .build();

        let mut wifi_storage = namespace;

        let ssid = wifi_storage.get::<String>("ssid").ok();
        let password = wifi_storage.get::<String>("password").ok();

        let mut ap_running = None;
        let mut sta_running = None;

        if let (Some(ssid), Some(password)) = (ssid, password) {
          let sta_config = StaConfig::builder()
            .ssid(&ssid)
            .password(&password)
            .build();

            let sta = wifi.into_sta(&sta_config);

            match sta.connect().await {
              Ok(sta) => {
                let StaRunning(ip) = sta;
                println!("Connected to '{}' with IP '{}'.", ssid, Ipv4Addr::from(ip));
                sta_running = Some(sta);
              },
              Err(err) => {
                let ap = err.wifi().into_ap(&ap_config);
                ap_running = Some(ap.start());
              }
            }
        } else {
          println!("Starting Access Point '{}' …", ap_ssid);
          let ap = wifi.into_ap(&ap_config);
          ap_running = Some(ap.start());
        }

        let stream = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 80)).expect("failed starting TCP listener");

        loop {
          match stream.accept() {
            Ok((client, addr)) => {
              match handle_request(client, addr, &ap_config, &mut wifi_storage, ap_running.take(), sta_running.take()).await {
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
      sleep(Duration::from_secs(5))
    }
  }
