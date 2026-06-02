use std::io::{self, Read};

pub struct ProgressReader<R, F>
where
  R: Read,
  F: FnMut(u64),
{
  inner: R,
  read: u64,
  on_progress: F,
}

impl<R, F> ProgressReader<R, F>
where
  R: Read,
  F: FnMut(u64),
{
  pub fn new(inner: R, on_progress: F) -> Self {
    Self {
      inner,
      read: 0,
      on_progress,
    }
  }
}

impl<R, F> Read for ProgressReader<R, F>
where
  R: Read,
  F: FnMut(u64),
{
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let n = self.inner.read(buf)?;
    if n > 0 {
      self.read += n as u64;
      (self.on_progress)(self.read);
    }
    Ok(n)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Read;

  #[test]
  fn test_progress_reader() {
    let data = b"hello world";
    let mut reader = ProgressReader::new(&data[..], |read| {
      assert!(read <= data.len() as u64);
    });

    let mut buf = [0u8; 5];
    let n = reader.read(&mut buf).unwrap();
    assert_eq!(n, 5);
    assert_eq!(reader.read, 5);

    let n = reader.read(&mut buf).unwrap();
    assert_eq!(n, 5);
    assert_eq!(reader.read, 10);

    let n = reader.read(&mut buf).unwrap();
    assert_eq!(n, 1);
    assert_eq!(reader.read, 11);

    let n = reader.read(&mut buf).unwrap();
    assert_eq!(n, 0);
    assert_eq!(reader.read, 11);
  }

  #[test]
  fn test_progress_reader_callback() {
    use std::cell::Cell;
    use std::rc::Rc;

    let data = b"abc";
    let total_read = Rc::new(Cell::new(0u64));
    let total_read_cb = total_read.clone();

    let mut reader = ProgressReader::new(&data[..], move |read| {
      total_read_cb.set(read);
    });

    let mut buf = [0u8; 1];
    reader.read(&mut buf).unwrap();
    assert_eq!(total_read.get(), 1);
    reader.read(&mut buf).unwrap();
    assert_eq!(total_read.get(), 2);
    reader.read(&mut buf).unwrap();
    assert_eq!(total_read.get(), 3);
    reader.read(&mut buf).unwrap();
    assert_eq!(total_read.get(), 3);
  }
}
