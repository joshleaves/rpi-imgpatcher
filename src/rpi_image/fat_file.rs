use crate::rpi_image::Error;
use fatfs::FileSystem;
use fscommon::BufStream;
use std::{
  fs::File,
  io::{Read, Seek, SeekFrom, Write},
};

// https://github.com/rafalh/rust-fatfs/blob/c4b88477b22ca7e5131fbd8891f62a5deaa88e6e/src/dir.rs#L97
// * wink wink *
fn split_path(path: &str) -> (&str, Option<&str>) {
  let trimmed_path = path.trim_matches('/');
  trimmed_path.find('/').map_or((trimmed_path, None), |n| {
    (&trimmed_path[..n], Some(&trimmed_path[n + 1..]))
  })
}

fn create_dir_r(fat: &FileSystem<BufStream<File>>, fat_path: &str) -> Result<(), Error> {
  let mut current = fat.root_dir();
  let remaining_path = fat_path.trim_matches('/');
  if remaining_path.is_empty() {
    return Ok(());
  }
  let mut remaining = remaining_path;

  while !remaining.is_empty() {
    let (name, rest) = split_path(remaining);

    current = match current.open_dir(name) {
      Ok(dir) => dir,
      Err(_) => current.create_dir(name)?,
    };
    match rest {
      Some(r) => remaining = r,
      None => break,
    }
  }

  Ok(())
}

pub fn read_file(fat: &FileSystem<BufStream<File>>, fat_path: &str) -> Result<Vec<u8>, Error> {
  let root_dir = fat.root_dir();
  let mut file = root_dir.open_file(fat_path)?;

  let mut buf = Vec::new();
  file.read_to_end(&mut buf)?;

  Ok(buf)
}

pub fn write_file(
  fat: &FileSystem<BufStream<File>>,
  fat_path: &str,
  external_file: &mut File,
) -> Result<u64, Error> {
  if let Some((dir, _)) = fat_path.rsplit_once('/') {
    create_dir_r(fat, dir)?;
  }

  let root_dir = fat.root_dir();
  let mut fat_file = root_dir.create_file(fat_path)?;
  let bytes_written = std::io::copy(external_file, &mut fat_file)?;

  Ok(bytes_written)
}

pub fn write_bytes(
  fat: &FileSystem<BufStream<File>>,
  fat_path: &str,
  bytes: &[u8],
) -> Result<u64, Error> {
  if let Some((dir, _)) = fat_path.rsplit_once('/') {
    create_dir_r(fat, dir)?;
  }

  let root_dir = fat.root_dir();
  let mut fat_file = root_dir.create_file(fat_path)?;
  fat_file.truncate()?;
  fat_file.write_all(bytes)?;

  Ok(bytes.len() as u64)
}

pub fn append_bytes(
  fat: &FileSystem<BufStream<File>>,
  fat_path: &str,
  bytes: &[u8],
) -> Result<u64, Error> {
  if let Some((dir, _)) = fat_path.rsplit_once('/') {
    create_dir_r(fat, dir)?;
  }

  let root_dir = fat.root_dir();
  let mut fat_file = match root_dir.open_file(fat_path) {
    Ok(f) => f,
    Err(_) => root_dir.create_file(fat_path)?,
  };
  fat_file.seek(SeekFrom::End(0))?;
  fat_file.write_all(bytes)?;

  Ok(bytes.len() as u64)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::rpi_image::FatPartitionLayout;
  use std::io::{SeekFrom, Write};
  use std::path::PathBuf;
  use tempfile::NamedTempFile;

  fn open_fat_from_fixture() -> (NamedTempFile, FileSystem<BufStream<File>>) {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("tests")
      .join("fixtures")
      .join("test.img");

    let mut image_file = File::open(&fixture_path).expect("should open fixture image");
    let layout = FatPartitionLayout::new(&mut image_file).expect("should read FAT layout");
    image_file
      .seek(SeekFrom::Start(layout.base))
      .expect("should seek to FAT start");

    let mut fat_tmp = NamedTempFile::new().expect("should create temp FAT workspace");
    std::io::copy(
      &mut (&mut image_file).take(layout.length),
      fat_tmp.as_file_mut(),
    )
    .expect("should copy FAT partition to temp workspace");
    fat_tmp
      .as_file_mut()
      .flush()
      .expect("should flush FAT workspace");
    fat_tmp
      .as_file_mut()
      .seek(SeekFrom::Start(0))
      .expect("should rewind FAT workspace");

    let fat_file = fat_tmp.reopen().expect("should reopen FAT workspace");
    let fat = FileSystem::new(BufStream::new(fat_file), fatfs::FsOptions::new())
      .expect("should mount FAT workspace");

    (fat_tmp, fat)
  }

  #[test]
  fn create_dir_r_is_idempotent_on_existing_path() {
    let (_fat_tmp, fat) = open_fat_from_fixture();

    create_dir_r(&fat, "boot/deep/a").expect("first create_dir_r call should succeed");
    create_dir_r(&fat, "boot/deep/a").expect("second create_dir_r call should also succeed");

    let root = fat.root_dir();
    root
      .open_dir("boot/deep/a")
      .expect("created directory should exist");
  }

  #[test]
  fn create_dir_r_supports_partially_existing_path() {
    let (_fat_tmp, fat) = open_fat_from_fixture();

    create_dir_r(&fat, "boot/partial").expect("should create first segment");
    create_dir_r(&fat, "boot/partial/more/depth")
      .expect("should extend when parent already exists");

    let root = fat.root_dir();
    root
      .open_dir("boot/partial/more/depth")
      .expect("extended path should exist");
  }

  #[test]
  fn write_bytes_creates_missing_subdirs_when_parent_exists() {
    let (_fat_tmp, fat) = open_fat_from_fixture();

    write_bytes(&fat, "boot/existing.txt", b"first")
      .expect("should write into existing parent directory");
    write_bytes(&fat, "boot/nested/deep/new.txt", b"second")
      .expect("should create nested directories under existing parent");

    let bytes = read_file(&fat, "boot/nested/deep/new.txt")
      .expect("should read back file written in nested path");
    assert_eq!(bytes, b"second");
  }
}
