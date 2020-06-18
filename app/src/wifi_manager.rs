use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};

use esp_idf_hal::{nvs::NameSpace, wifi::*};

fn ssid_and_password(params: &[u8]) -> (Option<Ssid>, Option<Password>) {
  let mut ssid = None;
  let mut password = None;

  for (name, value) in url::form_urlencoded::parse(&params) {
    match name.as_ref() {
      "ssid" => ssid = Ssid::from_bytes(value.as_bytes()).ok(),
      "password" => password = Password::from_bytes(value.as_bytes()).ok(),
      _ => if ssid.is_some() && password.is_some() { break },
    }
  }

  (ssid, password)
}

fn handle_index(mut client: TcpStream) -> io::Result<()> {
  writeln!(client, "HTTP/1.1 200 OK")?;
  writeln!(client, "Content-Type: text/html")?;
  writeln!(client)?;
  writeln!(client, "{}", include_str!("index.html"))
}

fn handle_hotspot_detect(mut client: TcpStream) -> io::Result<()> {
  writeln!(client, "HTTP/1.1 303 See Other")?;
  writeln!(client, "Location: http://192.168.4.1/")?;
  writeln!(client, "Content-Type: text/plain")?;
  writeln!(client)?;
  writeln!(client, "Redirecting …")
}

fn handle_connecting(mut client: TcpStream) -> io::Result<()> {
  writeln!(client, "HTTP/1.1 303 See Other")?;
  writeln!(client, "Location: http://192.168.4.1/")?;
  writeln!(client, "Content-Type: text/plain")?;
  writeln!(client)?;
  writeln!(client, "Connecting …")
}

fn handle_not_found(mut client: TcpStream) -> io::Result<()> {
  writeln!(client, "HTTP/1.1 404 Not Found")?;
  writeln!(client)
}

fn handle_internal_error(mut client: TcpStream) -> io::Result<()> {
  writeln!(client, "HTTP/1.1 500 INTERNAL SERVER ERROR")?;
  writeln!(client)
}

pub async fn handle_request(
  mut client: TcpStream, addr: SocketAddr,
  wifi_storage: &mut NameSpace,
  mut wifi_running: WifiRunning,
) -> WifiRunning {
  println!("Handling request from {} …", addr);

  let mut buf: [u8; 1024] = [0; 1024];
  let len = match client.read(&mut buf) {
    Ok(len) => len,
    Err(_) => return wifi_running,
  };

  let mut headers = [httparse::EMPTY_HEADER; 16];
  let mut req = httparse::Request::new(&mut headers);

  let status = req.parse(&buf);

  println!("Status: {:?}", status);
  println!("Request: {:?}", req);

  let res = if let Ok(httparse::Status::Complete(header_len)) = status {
    match req.path {
      Some("/") => handle_index(client),
      Some("/hotspot-detect.html") => handle_hotspot_detect(client),
      Some("/connect") => {
        let res = handle_connecting(client);

        if req.method == Some("POST") {
          let body = &buf[header_len..len];

          if let (Some(ssid), Some(password)) = ssid_and_password(body) {
            wifi_storage.set::<&str>("ssid", &ssid.as_str()).expect("Failed saving SSID");
            wifi_storage.set::<&str>("password", &password.as_str()).expect("Failed saving password");

            if let WifiRunning::Ap(ap) = wifi_running {
              let (ap_config, wifi) = ap.stop();
              wifi_running = connect_ssid_password(wifi, ap_config, ssid, password).await;
            }
          }
        }

        res
      },
      _ => handle_not_found(client),
    }
  } else {
    handle_internal_error(client)
  };

  if let Err(err) = res {
    eprintln!("Error handling request: {}", err);
  }

  wifi_running
}

pub async fn connect_ssid_password(wifi: Wifi<()>, ap_config: ApConfig, ssid: Ssid, password: Password) -> WifiRunning {
  let sta_config = StaConfig::builder()
    .ssid(ssid)
    .password(password)
    .build();

  println!("Connecting to '{}' with password '{}' …", sta_config.ssid(), sta_config.password());

  match wifi.connect_sta(sta_config).await {
    Ok(sta) => {
      if let WifiRunning::Sta(ref sta, ref ip_info) = sta {
        println!("Connected to '{}' with IP '{}'.", sta.config().ssid(), ip_info.ip());
      }
      sta
    },
    Err(err) => {
      err.wifi().start_ap(ap_config).unwrap()
    }
  }
}
