use crate::rpi_image::image_io::{copy_exact, copy_exact_n};
use crate::rpi_image::{Error, FatPartitionLayout};
use lzma_rust2::XzReaderMt;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

enum SourceImageState {
  Created,
  MbrHeaderRead,
  MbrRead,
  FatRead,
  Consumed,
}

pub enum SourceImageFile {
  Plain(File),
  Xz(Box<XzReaderMt<File>>),
}

pub struct SourceImageReader {
  state: SourceImageState,
  file: SourceImageFile,
}

impl SourceImageReader {
  pub fn new(source_image: PathBuf) -> Result<Self, Error> {
    let file = match source_image.extension().and_then(|e| e.to_str()) {
      Some("xz") => {
        let source_image = File::open(source_image)?;
        let source_image = XzReaderMt::<File>::new(source_image, true, 0)?;
        SourceImageFile::Xz(Box::new(source_image))
      }
      _ => {
        let source_image = File::open(source_image)?;
        SourceImageFile::Plain(source_image)
      }
    };
    Ok(Self {
      state: SourceImageState::Created,
      file,
    })
  }

  pub fn layout_fat(&mut self) -> Result<FatPartitionLayout, Error> {
    if !matches!(self.state, SourceImageState::Created) {
      return Err(Error::InvalidState);
    }

    let layout = match &mut self.file {
      SourceImageFile::Plain(source_image) => FatPartitionLayout::new(source_image),
      SourceImageFile::Xz(source_image) => {
        let mut mbr_sector = [0u8; 512];
        source_image.read_exact(&mut mbr_sector)?;
        let mut cursor = Cursor::new(mbr_sector);

        FatPartitionLayout::new(&mut cursor)
      }
    }?;

    self.state = SourceImageState::MbrHeaderRead;
    Ok(layout)
  }

  pub fn extract_fat_to_file(
    &mut self,
    layout: FatPartitionLayout,
    fat_tmp_file: &mut File,
  ) -> Result<(), Error> {
    if !matches!(self.state, SourceImageState::MbrHeaderRead) {
      return Err(Error::InvalidState);
    }

    match &mut self.file {
      SourceImageFile::Plain(file) => {
        file.seek(SeekFrom::Start(layout.base))?;
        copy_exact_n(file, fat_tmp_file, layout.length)
      }
      SourceImageFile::Xz(xz_reader_mt) => {
        let skipped = std::io::copy(
          &mut xz_reader_mt.take(layout.base - 512),
          &mut std::io::sink(),
        )?;
        if skipped != layout.base - 512 {
          return Err(Error::CopyMismatch);
        }
        copy_exact_n(xz_reader_mt, fat_tmp_file, layout.length)
      }
    }?;

    self.state = SourceImageState::FatRead;
    Ok(())
  }

  pub fn copy_mbr_to_file<W>(
    &mut self,
    layout: FatPartitionLayout,
    out_img: &mut W,
  ) -> Result<(), Error>
  where
    W: Write,
  {
    if !matches!(self.state, SourceImageState::Created) {
      return Err(Error::InvalidState);
    }

    match &mut self.file {
      SourceImageFile::Plain(file) => copy_exact_n(file, out_img, layout.base),
      SourceImageFile::Xz(xz_reader_mt) => copy_exact_n(xz_reader_mt, out_img, layout.base),
    }?;

    self.state = SourceImageState::MbrRead;
    Ok(())
  }

  pub fn skip_fat(&mut self, layout: FatPartitionLayout) -> Result<(), Error> {
    if !matches!(self.state, SourceImageState::MbrRead) {
      return Err(Error::InvalidState);
    }
    match &mut self.file {
      SourceImageFile::Plain(file) => file.seek(SeekFrom::Current(layout.length as i64))?,
      SourceImageFile::Xz(xz_reader_mt) => {
        let skipped = std::io::copy(&mut xz_reader_mt.take(layout.length), &mut std::io::sink())?;
        if skipped != layout.length {
          return Err(Error::CopyMismatch);
        }
        skipped
      }
    };

    self.state = SourceImageState::FatRead;
    Ok(())
  }

  pub fn copy_tail_to_file<W>(&mut self, out_img: &mut W) -> Result<(), Error>
  where
    W: Write,
  {
    if !matches!(self.state, SourceImageState::FatRead) {
      return Err(Error::InvalidState);
    }

    match &mut self.file {
      SourceImageFile::Plain(file) => copy_exact(file, out_img),
      SourceImageFile::Xz(xz_reader_mt) => copy_exact(xz_reader_mt, out_img),
    }?;

    self.state = SourceImageState::Consumed;
    Ok(())
  }
}
