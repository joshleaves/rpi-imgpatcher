use crate::rpi_image::Error;
use std::io::{Read, Write};

#[cfg(feature = "buffered_copy")]
const DEFAULT: usize = 4 * 1024 * 1024;
#[cfg(feature = "buffered_copy")]
const BUFFER_SIZE: usize = match option_env!("RPI_BUFFER_SIZE") {
  Some(val) => match val.parse::<usize>() {
    Ok(v) => v,
    Err(_) => DEFAULT,
  },
  None => DEFAULT,
};


#[cfg(not(feature = "buffered_copy"))]
pub fn copy_exact<R, W>(src: &mut R, dst: &mut W) -> Result<u64, Error>
where
  R: Read,
  W: Write,
{
  std::io::copy(src, dst).map_err(Error::Io)
}

#[cfg(feature = "buffered_copy")]
pub fn copy_exact<R, W>(src: &mut R, dst: &mut W) -> Result<u64, Error>
where
  R: Read,
  W: Write,
{
  let mut buf = vec![0u8; BUFFER_SIZE];
  let mut total = 0;

  loop {
    let n = src.read(&mut buf)?;
    if n == 0 {
      break;
    }

    dst.write_all(&buf[..n])?;
    total += n as u64;
  }

  Ok(total)
}

#[cfg(not(feature = "buffered_copy"))]
pub fn copy_exact_n<R, W>(src: &mut R, dst: &mut W, length: u64) -> Result<u64, Error>
where
  R: Read,
  W: Write,
{
  let mut limited = src.take(length);
  let copied_len = std::io::copy(&mut limited, dst)?;
  if copied_len != length {
    return Err(Error::CopyMismatch);
  }

  Ok(copied_len)
}

#[cfg(feature = "buffered_copy")]
pub fn copy_exact_n<R, W>(src: &mut R, dst: &mut W, length: u64,) -> Result<u64, Error>
where
  R: Read,
  W: Write,
{
  let mut limited = src.take(length);
  let mut buf = vec![0u8; BUFFER_SIZE];
  let mut total = 0;

  loop {
    let n = limited.read(&mut buf)?;
    if n == 0 {
      break;
    }

    dst.write_all(&buf[..n])?;
    total += n as u64;
  }

  if total != length {
    return Err(Error::CopyMismatch);
  }

  Ok(total)
}