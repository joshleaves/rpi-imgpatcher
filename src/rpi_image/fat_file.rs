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
  let mut remaining = fat_path.trim_matches('/');

  while !remaining.is_empty() {
    let (name, rest) = split_path(remaining);

    current = current.create_dir(name)?;
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
