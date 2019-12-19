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

include!("wifi_credentials.rs");

use embedded_hal::digital::v2::OutputPin;

use esp32_hal::{EspError, gpio::*, nvs::*, wifi::{Wifi, WifiRunning, StaConfig}};

use core::ptr;
use esp_idf_sys::*;

const UART_NUM: uart_port_t = uart_port_t::UART_NUM_0;
const ECHO_TEST_TXD: i32 = gpio_num_t::GPIO_NUM_17 as i32;
const ECHO_TEST_RXD: i32 = gpio_num_t::GPIO_NUM_16 as i32;
const ECHO_TEST_RTS: i32 = UART_PIN_NO_CHANGE;
const ECHO_TEST_CTS: i32 = UART_PIN_NO_CHANGE;

const BUF_SIZE: i32 = 1024;

#[no_mangle]
pub fn app_main() {
  executor::spawn(async {
    if let Err(err) = rust_blink_and_write().await {
      println!("{}", err);
    }
  })
}

async fn rust_blink_and_write() -> Result<!, EspError> {
  let mut gpio = GPIO22::into_input_output();

    // Configure parameters of an UART driver,
    // communication pins and install the driver.
    let uart_config = uart_config_t {
        baud_rate: 115200,
        data_bits: uart_word_length_t::UART_DATA_8_BITS,
        parity: uart_parity_t::UART_PARITY_DISABLE,
        stop_bits: uart_stop_bits_t::UART_STOP_BITS_1,
        flow_ctrl: uart_hw_flowcontrol_t::UART_HW_FLOWCTRL_DISABLE,
        rx_flow_ctrl_thresh: 0,
        use_ref_tick: false,
    };

    unsafe {
      uart_param_config(UART_NUM, &uart_config);
      uart_set_pin(UART_NUM, ECHO_TEST_TXD, ECHO_TEST_RXD, ECHO_TEST_RTS, ECHO_TEST_CTS);
      uart_driver_install(UART_NUM, BUF_SIZE * 2, 0, 0, ptr::null_mut(), 0);
    }

    let mut nvs = NonVolatileStorage::default()?;

    let wifi = Wifi::init(&mut nvs);

    // esp32_hal::wifi::wifi_scan(true, false, 1000)?;

    let sta_config = StaConfig::builder()
      .ssid(SSID)
      .password(PASSWORD)
      .build();

    let sta = wifi.into_sta(&sta_config);

    let wifi_running = sta.connect().await;

    match wifi_running {
      Ok(WifiRunning(ip)) => {
        println!("Connected with IP {}.", Ipv4Addr::from(ip));
      },
      Err(e) => {
        println!("Failed connecting to WiFi: {}", e);
      }
    }

    let mutex = Mutex::new(0usize);
    println!("mutex value = {:?}", *mutex.lock().unwrap());
    *mutex.lock().unwrap() = 1;
    println!("mutex value = {:?}", *mutex.lock().unwrap());

    let rwlock = RwLock::new(0usize);
    println!("rwlock value = {:?}", *rwlock.read().unwrap());
    *rwlock.write().unwrap() = 1;
    println!("rwlock value = {:?}", *rwlock.read().unwrap());

    let stream = std::net::TcpListener::bind("0.0.0.0:80").expect("failed starting TCP listener");

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

    thread::spawn(move || {
      loop {
        gpio.set_low().unwrap();
        sleep(Duration::from_millis(100));
        gpio.set_high().unwrap();
        sleep(Duration::from_secs(3));
      }
    });

    loop {
      if let Ok(ref mut namespace) = namespace {
        println!("GOT VALUE: {:?}", namespace.get::<i32>("forty-two").map_err(|e| e.to_string()));
        println!("GOT VALUE: {:?}", namespace.get::<String>("hakuna").map_err(|e| e.to_string()));

        println!("Setting value: {:?}", namespace.set::<i32>("forty-two", 42).map_err(|e| e.to_string()));
        println!("Setting value: {:?}", namespace.set::<&str>("hakuna", "matata").map_err(|e| e.to_string()));
      }

      match stream.accept() {
        Ok((mut client, addr)) => {
          println!("new client: {:?}", addr);

          let mut buf: [u8; 1024] = [0; 1024];
          client.read(&mut buf).unwrap();

          let mut headers = [httparse::EMPTY_HEADER; 16];
          let mut req = httparse::Request::new(&mut headers);
          let res = req.parse(&buf);

          println!("Request: {:?}", res);

          writeln!(client, "HTTP/1.1 200 OK").unwrap();
          writeln!(client).unwrap();
          writeln!(client, "Hello, world!").unwrap();
        },
        Err(e) => {
          if e.kind() != std::io::ErrorKind::WouldBlock {
            println!("couldn't get client: {:?}", e);
          }
        },
      }

      thread::yield_now();
    }
  }
