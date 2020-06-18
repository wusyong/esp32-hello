#![feature(never_type)]
#![no_main]

extern crate alloc;

use alloc::string::String;

#[macro_use]
extern crate std;

use std::thread::{self, sleep};
use std::time::Duration;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::net::TcpListener;

use embedded_hal::digital::v2::OutputPin;
use macaddr::MacAddr;

use esp_idf_hal::{*, nvs::*, wifi::*};

use futures::executor::block_on;

mod wifi_manager;
use wifi_manager::*;

mod dns;

#[no_mangle]
pub fn app_main() {
  block_on(async {
    if let Err(err) = rust_blink_and_write().await {
      println!("{}", err);
    }
  })
}

async fn rust_blink_and_write() -> Result<!, EspError> {
    use esp32_hal::{target, gpio::GpioExt};

    let dp = unsafe { target::Peripherals::steal() };
    let pins = dp.GPIO.split();

    let mut gpio = pins.gpio22.into_open_drain_output();

    let mut nvs = NonVolatileStorage::default();

    let wifi = Wifi::init(&mut nvs)?;

    println!("AP started.");

    // esp32_hal::wifi::wifi_scan(true, false, 1000)?;

    let namespace = nvs.open("wifi")?;
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
        let mac = MacAddr::from(MacAddrType::Ap);

        let ap_ssid = Ssid::from_bytes(format!("ESP {}", mac).as_bytes()).unwrap();

        let ap_config = ApConfig::builder()
          .ssid(ap_ssid)
          .build();

        let mut wifi_storage = namespace;

        let ssid = wifi_storage.get::<String>("ssid").ok().and_then(|s| Ssid::from_bytes(s.as_bytes()).ok());
        let password = wifi_storage.get::<String>("password").ok().and_then(|s| Password::from_bytes(s.as_bytes()).ok());

        let mut wifi_running;

        if let (Some(ssid), Some(password)) = (ssid, password) {
          let sta_config = StaConfig::builder()
            .ssid(ssid)
            .password(password)
            .build();

            let sta = wifi.into_sta(&sta_config).unwrap();

            match sta.connect().await {
              Ok(sta) => {
                if let WifiRunning::Sta(ip) = sta {
                  println!("Connected to '{}' with IP '{}'.", sta_config.ssid(), Ipv4Addr::from(ip));
                }
                wifi_running = sta;
              },
              Err(err) => {
                wifi_running = err.wifi().into_ap(&ap_config).unwrap().start();
              }
            }
        } else {
          println!("Starting Access Point '{}' …", ap_config.ssid());
          wifi_running = wifi.into_ap(&ap_config).unwrap().start();
        }

        let stream = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 80)).expect("failed starting TCP listener");

        loop {
          match stream.accept() {
            Ok((client, addr)) => {
              match handle_request(client, addr, &ap_config, &mut wifi_storage, wifi_running).await {
                Ok(wr) => {
                  wifi_running = wr;
                },
                Err(err) => {
                  panic!("Failed to handle request: {:?}", err);
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
