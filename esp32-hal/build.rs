use std::env;

fn main() {
  let target_device = match env::var("TARGET").expect("TARGET not set").as_ref() {
    "xtensa-esp32-none-elf" => "esp32",
    "xtensa-esp8266-none-elf" => "esp8266",
    _ => return,
  };

  println!(r#"cargo:rustc-cfg=target_device="{}""#, target_device);
}
