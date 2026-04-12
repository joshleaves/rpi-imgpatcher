use crate::rpi_image::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

// Attention: no safeguards here, you are responsible for what you copy
pub fn extract_fat_to_file(
  input_file: &mut File,
  output_file: &mut File,
  base: u64,
  length: u64,
) -> Result<(), Error> {
  input_file.seek(SeekFrom::Start(base))?;
  let mut limited = input_file.take(length);
  let copied_len = std::io::copy(&mut limited, output_file)?;

  if copied_len != length {
    return Err(Error::CopyMismatch);
  }

  output_file.seek(SeekFrom::Start(0))?;
  Ok(())
}
