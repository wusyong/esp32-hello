#![no_main]

use std::time::Duration;
use std::thread::sleep;

use futures::executor::block_on;

use esp_idf_hal::{nvs::*, wifi::*};

#[no_mangle]
fn app_main() {
  block_on(async {
    let mut nvs = NonVolatileStorage::default();
    let mut wifi = Wifi::init(&mut nvs).unwrap();

    loop {
      if let Ok(aps) = wifi.scan().await {
        println!("Found {} access points:", aps.len());

        for ap in aps {
          println!("  - {} '{}'", ap.bssid(), ap.ssid())
        }
      } else {
        println!("No access points found.");
      }

      sleep(Duration::from_secs(5));
    }
  })
}
