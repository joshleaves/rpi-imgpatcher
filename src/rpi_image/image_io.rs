use crate::rpi_image::Error;
use std::io::{Read, Write};

pub fn copy_exact<R, W>(src: &mut R, dst: &mut W) -> Result<u64, Error>
where
  R: Read,
  W: Write,
{
  std::io::copy(src, dst).map_err(Error::Io)
}

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
