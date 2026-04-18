use std::io::{self, Write};

pub struct ProgressWriter<W, F>
where
  W: Write,
  F: FnMut(u64),
{
  inner: W,
  written: u64,
  on_progress: F,
}

impl<W, F> ProgressWriter<W, F>
where
  W: Write,
  F: FnMut(u64),
{
  pub fn new(inner: W, on_progress: F) -> Self {
    Self {
      inner,
      written: 0,
      on_progress,
    }
  }

  pub fn inner(self) -> W {
    self.inner
  }
}

impl<W, F> Write for ProgressWriter<W, F>
where
  W: Write,
  F: FnMut(u64),
{
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let n = self.inner.write(buf)?;
    self.written += n as u64;
    (self.on_progress)(self.written);
    Ok(n)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.inner.flush()
  }

  fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
    self.inner.write_all(buf)?;
    self.written += buf.len() as u64;
    (self.on_progress)(self.written);
    Ok(())
  }
}
