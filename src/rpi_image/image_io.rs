use crate::rpi_image::Error;
use std::io::{Read, Write};

#[cfg(feature = "buffered_copy")]
const DEFAULT_COPY_BUFFER_SIZE: usize = 4 * 1024 * 1024;
#[cfg(feature = "buffered_copy")]
const COPY_BUFFER_SIZE: usize = match option_env!("RPI_COPY_BUFFER_SIZE") {
  Some(val) => match usize::from_str_radix(val, 10) {
    Ok(v) => v,
    Err(_) => DEFAULT_COPY_BUFFER_SIZE,
  },
  None => DEFAULT_COPY_BUFFER_SIZE,
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
  let mut buf = vec![0u8; COPY_BUFFER_SIZE];
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
pub fn copy_exact_n<R, W>(src: &mut R, dst: &mut W, length: u64) -> Result<u64, Error>
where
  R: Read,
  W: Write,
{
  let mut limited = src.take(length);
  let mut buf = vec![0u8; COPY_BUFFER_SIZE];
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

const DEFAULT_COMPARE_BUFFER_SIZE: usize = 64 * 1024;
const COMPARE_BUFFER_SIZE: usize = match option_env!("RPI_COMPARE_BUFFER_SIZE") {
  Some(val) => match usize::from_str_radix(val, 10) {
    Ok(v) => v,
    Err(_) => DEFAULT_COMPARE_BUFFER_SIZE,
  },
  None => DEFAULT_COMPARE_BUFFER_SIZE,
};

pub fn compare<R1, R2>(lhs: &mut R1, rhs: &mut R2) -> Result<bool, Error>
where
  R1: Read,
  R2: Read,
{
  let mut buf1 = [0u8; COMPARE_BUFFER_SIZE];
  let mut buf2 = [0u8; COMPARE_BUFFER_SIZE];

  loop {
    let n1 = lhs.read(&mut buf1)?;
    // LHS reaches EOF, check if RHS also reaches EOF
    if n1 == 0 {
      let mut buf_extra = [0u8; 1];
      return Ok(rhs.read(&mut buf_extra)? == 0);
    }

    // We try to read just as many bytes from RHS
    let mut n2 = 0;
    while n2 < n1 {
      match rhs.read(&mut buf2[n2..n1]) {
        Ok(0) => return Ok(false),
        Ok(n) => n2 += n,
        Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
        Err(err) => return Err(err.into()),
      }
    }

    // Let's compare stuff
    if buf1[..n1] != buf2[..n1] {
      return Ok(false);
    }
  }
}

#[cfg(test)]
mod image_io_tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_compare_different_lengths() {
        let mut lhs = Cursor::new(vec![1, 2, 3]);
        let mut rhs = Cursor::new(vec![1, 2, 3, 4]);
        assert!(!compare(&mut lhs, &mut rhs).unwrap());

        let mut lhs = Cursor::new(vec![1, 2, 3, 4]);
        let mut rhs = Cursor::new(vec![1, 2, 3]);
        assert!(!compare(&mut lhs, &mut rhs).unwrap());

        let mut lhs = Cursor::new(vec![1, 2, 3]);
        let mut rhs = Cursor::new(vec![1, 2, 3]);
        assert!(compare(&mut lhs, &mut rhs).unwrap());
    }
}
