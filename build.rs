use std::{env, error::Error, fs::{remove_file, File}, io::stderr, os::unix::{fs::symlink, io::{FromRawFd, AsRawFd}}, path::PathBuf, process::{Command, Stdio}};

fn main() -> Result<(), Box<dyn Error>> {
  let target_dir = PathBuf::from(env::var("CARGO_TARGET_DIR")?);

  let idf_path = PathBuf::from(env::var("IDF_PATH")?);
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
    .arg("app")
    .stdout(Stdio::from(stderr.try_clone()?))
    .stderr(Stdio::from(stderr.try_clone()?))
    .status()?;

  assert!(status.success());

  Ok(())
}
