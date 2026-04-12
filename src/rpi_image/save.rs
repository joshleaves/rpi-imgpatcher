use crate::RpiImage;
use crate::rpi_image::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub enum SaveStrategy {
  ToFile(PathBuf),
  Overwrite,
}

impl SaveStrategy {
  pub fn save(&self, rpi: RpiImage) -> Result<(), Error> {
    match self {
      SaveStrategy::ToFile(file) => self.save_to_file(rpi, file),
      SaveStrategy::Overwrite => self.overwrite_in_place(rpi),
    }
  }

  fn save_to_file(&self, rpi: RpiImage, file: &Path) -> Result<(), Error> {
    let RpiImage { fat, .. } = rpi;
    std::mem::drop(fat);
    let mut src = File::open(&rpi.image_path)?;
    let mut fat = File::open(&rpi.fat_tmp_path)?;
    let mut dst = OpenOptions::new()
      .create(true)
      .truncate(true)
      .read(true)
      .write(true)
      .open(file)?;

    let mut header = (&mut src).take(rpi.fat_base);
    let bytes_written_header = std::io::copy(&mut header, &mut dst)?;
    if bytes_written_header != rpi.fat_base {
      return Err(Error::CopyMismatch);
    }

    let bytes_written_fat = std::io::copy(&mut fat, &mut dst)?;
    if bytes_written_fat != rpi.fat_length {
      return Err(Error::CopyMismatch);
    }

    src.seek(SeekFrom::Current(rpi.fat_length as i64))?;
    let bytes_written_ext4 = std::io::copy(&mut src, &mut dst)?;
    let image_len = src.metadata()?.len();
    let expected_ext4_len = image_len - (rpi.fat_base + rpi.fat_length);
    if bytes_written_ext4 != expected_ext4_len {
      return Err(Error::CopyMismatch);
    }

    Ok(())
  }

  fn overwrite_in_place(&self, rpi: RpiImage) -> Result<(), Error> {
    let RpiImage { fat, .. } = rpi;
    std::mem::drop(fat);

    let mut src = File::open(&rpi.fat_tmp_path)?;
    let mut dst = OpenOptions::new()
      .read(true)
      .write(true)
      .open(&rpi.image_path)?;
    dst.seek(SeekFrom::Start(rpi.fat_base))?;

    let bytes_written = std::io::copy(&mut src, &mut dst)?;
    if bytes_written != rpi.fat_length {
      return Err(Error::CopyMismatch);
    }
    dst.flush()?;

    Ok(())
  }
}
