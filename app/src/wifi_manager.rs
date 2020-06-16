use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::borrow::Cow;

use esp_idf_hal::{nvs::NameSpace, wifi::*};

fn ssid_and_password(params: &[u8]) -> (Option<Cow<str>>, Option<Cow<str>>) {
  let mut ssid = None;
  let mut password = None;

  for (name, value) in url::form_urlencoded::parse(&params) {
    match name.as_ref() {
      "ssid" => ssid = Some(value),
      "password" => password = Some(value),
      _ => if ssid.is_some() && password.is_some() { break },
    }
  }

  (ssid, password)
}

pub async fn handle_request(
  mut client: TcpStream, addr: SocketAddr,
  ap_config: &ApConfig,
  wifi_storage: &mut NameSpace,
  mut ap_running: Option<ApRunning>, mut sta_running: Option<StaRunning>,
) -> io::Result<(Option<ApRunning>, Option<StaRunning>)> {
  println!("Handling request from {} …", addr);

  let mut buf: [u8; 1024] = [0; 1024];
  let len = client.read(&mut buf).unwrap();

  let mut headers = [httparse::EMPTY_HEADER; 16];
  let mut req = httparse::Request::new(&mut headers);

  let status = req.parse(&buf);

  println!("Status: {:?}", status);
  println!("Request: {:?}", req);

  if let Ok(httparse::Status::Complete(res)) = status {

    match req.path {
      Some("/") => {
        writeln!(client, "HTTP/1.1 200 OK")?;
        writeln!(client, "Content-Type: text/html")?;
        writeln!(client)?;
        writeln!(client, "{}", include_str!("index.html"))?;
      },
      Some("/hotspot-detect.html") => {
        writeln!(client, "HTTP/1.1 303 See Other")?;
        writeln!(client, "Location: http://192.168.4.1/")?;
        writeln!(client, "Content-Type: text/plain")?;
        writeln!(client)?;
        writeln!(client, "Redirecting …")?;
      },
      Some("/connect") => {
        writeln!(client, "HTTP/1.1 303 See Other")?;
        writeln!(client, "Location: http://192.168.4.1/")?;
        writeln!(client, "Content-Type: text/plain")?;
        writeln!(client)?;
        writeln!(client, "Connecting …")?;
        drop(client);

        match req.method {
          Some("POST") => {
            let body = &buf[res..len];

            if let (Some(ssid), Some(password)) = ssid_and_password(body) {
              wifi_storage.set::<&str>("ssid", &ssid).expect("Failed saving SSID");
              wifi_storage.set::<&str>("password", &password).expect("Failed saving password");

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
