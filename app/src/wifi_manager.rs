use std::future::Future;
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};

use esp32_hal::wifi::*;

pub async fn handle_request(
  mut client: TcpStream, addr: SocketAddr,
  ap_config: &ApConfig,
  mut ap_running: Option<ApRunning>, mut sta_running: Option<StaRunning>,
) -> io::Result<(Option<ApRunning>, Option<StaRunning>)> {
  println!("Handling request from {} …", addr);

  let mut buf: [u8; 1024] = [0; 1024];
  let len = client.read(&mut buf).unwrap();

  let mut headers = [httparse::EMPTY_HEADER; 16];
  let mut req = httparse::Request::new(&mut headers);

  if let Ok(httparse::Status::Complete(res)) = req.parse(&buf) {
    println!("Request: {:?}", req);

    match req.path {
      Some("/") => {
        writeln!(client, "HTTP/1.1 200 OK")?;
        writeln!(client, "Content-Type: text/html")?;
        writeln!(client)?;
        writeln!(client, include_str!("index.html"))?;
      },
      Some("/connect") => {
        writeln!(client, "HTTP/1.1 303 See Other")?;
        writeln!(client, "Location: /")?;
        writeln!(client)?;

        match req.method {
          Some("POST") => {
            let mut ssid = None;
            let mut password = None;

            for (name, value) in url::form_urlencoded::parse(&buf[res..len]) {
              match name.as_ref() {
                "ssid" => ssid = Some(value),
                "password" => password = Some(value),
                _ => if ssid.is_some() && password.is_some() { break },
              }
            }

            if let (Some(ssid), Some(password)) = (ssid, password) {
              if let Some(wifi) = ap_running.take().map(|w| w.stop()) {
                let sta_config = StaConfig::builder()
                  .ssid(&ssid)
                  .password(&password)
                  .build();

                println!("Connecting to '{}' with password '{}' …", ssid, password);

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
              }
            }
          },
          _ => {},
        }
      },
      _ => {
        writeln!(client, "HTTP/1.1 404 Not Found")?;
        writeln!(client)?;
      },
    }
  } else {
    writeln!(client, "HTTP/1.1 500 INTERNAL SERVER ERROR")?;
    writeln!(client)?;
  }

  Ok((ap_running, sta_running))
}
