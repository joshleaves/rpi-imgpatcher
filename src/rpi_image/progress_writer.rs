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

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;

  #[test]
  fn test_progress_writer() {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut dest = Vec::new();
    let total_written = Rc::new(Cell::new(0u64));
    let total_written_cb = total_written.clone();

    {
      let mut writer = ProgressWriter::new(&mut dest, move |written| {
        total_written_cb.set(written);
      });

      writer.write_all(b"hello").unwrap();
      assert_eq!(total_written.get(), 5);

      writer.write_all(b" ").unwrap();
      assert_eq!(total_written.get(), 6);

      writer.write_all(b"world").unwrap();
      assert_eq!(total_written.get(), 11);
    }

    assert_eq!(dest, b"hello world");
    assert_eq!(total_written.get(), 11);
  }
}
