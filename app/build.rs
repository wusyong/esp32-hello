use std::{env, error::Error, fs::{remove_file, File}, io::stderr, os::unix::{fs::symlink, io::{FromRawFd, AsRawFd}}, path::PathBuf, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
  println!("cargo:rerun-if-changed=Makefile");
  println!("cargo:rerun-if-changed=components/compiler_builtins/atomics.c");
  println!("cargo:rerun-if-changed=components/compiler_builtins/component.mk");
  println!("cargo:rerun-if-changed=main/app_main.c");
  println!("cargo:rerun-if-changed=main/component.mk");
  println!("cargo:rerun-if-changed=partitions.csv");
  println!("cargo:rerun-if-changed=sdkconfig");

  let target_dir = PathBuf::from(env::var("CARGO_TARGET_DIR").expect("CARGO_TARGET_DIR is unset"));

  let idf_path = PathBuf::from(env::var("IDF_PATH").expect("IDF_PATH is unset"));
  let idf_link = target_dir.join("esp-idf");

  let create_idf_symlink = || symlink(&idf_path, &idf_link);
  if let Ok(link_path) = idf_link.read_link() {
    if link_path != idf_path {
      remove_file(&idf_link)?;
      create_idf_symlink()?;
    }
  } else {
    create_idf_symlink()?;
  }

  let stderr = unsafe { File::from_raw_fd(stderr().as_raw_fd()) };
  let status = Command::new("make")
    .arg("-j")
    .arg("bootloader")
    .env("VERBOSE", "1")
    .stdout(stderr.try_clone()?)
    .stderr(stderr.try_clone()?)
    .status()?;

  assert!(status.success());

  let status = Command::new("make")
    .arg("-j")
    .arg("app")
    .env("VERBOSE", "1")
    .stdout(stderr.try_clone()?)
    .stderr(stderr.try_clone()?)
    .status()?;

  assert!(status.success());

  Ok(())
}
