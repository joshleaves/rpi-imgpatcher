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
