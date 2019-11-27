use std::{env, error::Error, fs::File, io::stderr, os::unix::io::{FromRawFd, AsRawFd}, path::PathBuf, process::{Command, Stdio}};

fn main() -> Result<(), Box<dyn Error>> {
  println!("cargo:rerun-if-changed=src/bindings.h");
  println!("cargo:rerun-if-changed=sdkconfig");

  let target_dir = PathBuf::from(env::var("CARGO_TARGET_DIR")?);

  let esp_path = PathBuf::from(env::var("ESP_PATH")?);
  let idf_path = PathBuf::from(env::var("IDF_PATH")?);

  let stderr = unsafe { File::from_raw_fd(stderr().as_raw_fd()) };
  let status = Command::new("make")
    .arg("-j")
    .arg("app")
    .stdout(Stdio::from(stderr.try_clone()?))
    .stderr(Stdio::from(stderr.try_clone()?))
    .status()?;

  assert!(status.success());

  let esp_sysroot = esp_path.join("xtensa-esp32-elf").join("xtensa-esp32-elf").join("sysroot");

  let includes =
    globwalk::GlobWalkerBuilder::from_patterns(
      &idf_path,
      &[
        "components/*/include",
        "components/*/platform_include",
        "components/**/esp32/include",
        "components/lwip/lwip/src/include",
        "components/lwip/include/apps",
      ],
    )
    .build()?
    .into_iter()
    .filter_map(Result::ok)
    .map(|include| format!("-I{}", include.into_path().display()))
    .collect::<Vec<_>>();

  let bindings = bindgen::Builder::default()
    .use_core()
    .layout_tests(false)
    .ctypes_prefix("libc")
    .header("src/bindings.h")
    .clang_arg(format!("--sysroot={}", esp_sysroot.display()))
    .clang_arg(format!("-I{}", target_dir.join("esp-build").join("include").display()))
    .clang_arg("-D__bindgen")
    .clang_args(&["-target", "xtensa"])
    .clang_args(&["-x", "c"])
    .clang_args(includes);

  eprintln!("{:?}", bindings.command_line_flags());

  let out_path = PathBuf::from(env::var("OUT_DIR")?);
  bindings.generate()
    .expect("Failed to generate bindings")
    .write_to_file(out_path.join("bindings.rs"))?;

  Ok(())
}
