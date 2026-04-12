use fatfs::FileSystem;
use fscommon::BufStream;
use lzma_rust2::{XzOptions, XzWriterMt};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
mod layout;
pub use layout::FatPartitionLayout;
mod error;
pub use error::Error;
mod fat_file;
mod image_io;
mod source_image;
use source_image::SourceImageReader;

pub struct RpiImage {
  // Path to the original disk image.
  // The original image is never modified.
  image_path: PathBuf,

  // Byte range of the boot FAT partition inside the source image.
  layout: FatPartitionLayout,

  // Path to the temporary working copy of the FAT partition.
  fat_tmp_path: PathBuf,

  // Open FAT workspace backed by the extracted temporary file.
  fat: FileSystem<BufStream<File>>,
}

impl RpiImage {
  /// Open an image editing session for a given boot FAT partition.
  ///
  /// This extracts the FAT partition to a temporary working file that becomes
  /// the mutable workspace for all subsequent operations.
  pub fn new(image_path: impl AsRef<Path>) -> Result<Self, Error> {
    let image_path = image_path.as_ref().to_path_buf();

    let mut image_file = SourceImageReader::new(image_path.clone())?;
    let layout = image_file.layout_fat()?;

    let fat_tmp = NamedTempFile::new()?;
    let (mut fat_tmp_file, fat_tmp_path) = fat_tmp.keep()?;
    image_file.extract_fat_to_file(layout, &mut fat_tmp_file)?;

    fat_tmp_file.seek(SeekFrom::Start(0))?;
    let buf_stream = BufStream::new(fat_tmp_file);
    let fat = fatfs::FileSystem::new(buf_stream, fatfs::FsOptions::new())?;

    Ok(Self {
      image_path,

      layout,

      fat_tmp_path,
      fat,
    })
  }

  /// Read a file from the extracted FAT workspace.
  ///
  /// The returned bytes reflect the current session state,
  /// including any prior writes that have not yet been saved back to the
  /// source image.
  pub fn read_file(&self, fat_path: &str) -> Result<Vec<u8>, Error> {
    fat_file::read_file(&self.fat, fat_path)
  }

  /// Create or replace a file in the extracted FAT workspace using the
  /// contents of a file from the host filesystem.
  pub fn write_file(&mut self, fat_path: &str, file: impl AsRef<Path>) -> Result<u64, Error> {
    let mut file = File::open(file)?;
    fat_file::write_file(&self.fat, fat_path, &mut file)
  }

  /// Create or replace a file in the extracted FAT workspace using raw bytes.
  pub fn write_bytes(&mut self, fat_path: &str, bytes: &[u8]) -> Result<u64, Error> {
    fat_file::write_bytes(&self.fat, fat_path, bytes)
  }

  /// Append bytes to a file in the extracted FAT workspace.
  ///
  /// If the file does not exist, it is created first.
  pub fn append_bytes(&mut self, fat_path: &str, bytes: &[u8]) -> Result<u64, Error> {
    fat_file::append_bytes(&self.fat, fat_path, bytes)
  }

  /// Rebuild the full disk image into a new output file.
  ///
  /// The original image remains untouched.
  pub fn save_to_file(self, out_file: impl AsRef<Path>) -> Result<(), Error> {
    let path = out_file.as_ref();

    let mut writer = OpenOptions::new()
      .create(true)
      .truncate(true)
      .read(true)
      .write(true)
      .open(path)?;

    match path.extension().and_then(|e| e.to_str()) {
      Some("xz") => {
        let opts = XzOptions {
          block_size: Some(NonZeroU64::new(4 * 1024 * 1024).unwrap()),
          ..Default::default()
        };
        let mut writer = XzWriterMt::<File>::new(writer, opts, 0)?;
        self.save_to_writer(&mut writer)?;
        writer.finish()?;
        Ok(())
      }
      _ => self.save_to_writer(&mut writer),
    }?;
    Ok(())
  }

  fn save_to_writer<W>(self, writer: &mut W) -> Result<(), Error>
  where
    W: Write,
  {
    let RpiImage {
      fat,
      layout,
      image_path,
      fat_tmp_path,
      ..
    } = self;
    std::mem::drop(fat);

    let mut image_file = SourceImageReader::new(image_path.clone())?;
    let mut fat = File::open(fat_tmp_path)?;
    image_file.copy_mbr_to_file(layout, writer)?;
    image_file.skip_fat(layout)?;
    image_io::copy_exact_n(&mut fat, writer, layout.length)?;
    image_file.copy_tail_to_file(writer)?;
    writer.flush()?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crc32fast::hash;
  use std::fs::File;
  use std::io::{Cursor, Read, Seek, SeekFrom};
  use uuid::Uuid;

  #[test]
  fn save_to_file_can_be_verified_by_reading_back_the_raw_fat_image() {
    // First, let's make up random data
    let uuid = Uuid::new_v4().to_string();
    let file_name = format!("{:08x}.txt", hash(uuid.as_bytes()));

    // Open our fixture IMG
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("tests")
      .join("fixtures")
      .join("test.img");

    // Write our random data
    let mut image = RpiImage::new(&fixture_path).expect("should open fixture image");
    image
      .write_bytes(&file_name, uuid.as_bytes())
      .expect("should write bytes into FAT workspace");

    // Âaaand save
    let output = NamedTempFile::new().expect("should create output temp file");
    let output_path = output.path().to_path_buf();
    image
      .save_to_file(&output_path)
      .expect("should save patched image to temp file");

    // Re-open and get the FAT
    let mut output_file = File::open(&output_path).expect("should reopen saved image");
    let layout = FatPartitionLayout::new(&mut output_file).expect("should read FAT layout");

    output_file
      .seek(SeekFrom::Start(layout.base))
      .expect("should seek to FAT start");
    let mut fat_bytes = Vec::with_capacity(layout.length as usize);
    (&mut output_file)
      .take(layout.length)
      .read_to_end(&mut fat_bytes)
      .expect("should read extracted FAT bytes");

    // Now open the FAT with fatfs-rs
    let fat_stream = BufStream::new(Cursor::new(fat_bytes));
    let fat_fs = fatfs::FileSystem::new(fat_stream, fatfs::FsOptions::new())
      .expect("oracle should open FAT filesystem from RAM buffer");

    let root_dir = fat_fs.root_dir();
    let mut oracle_file = root_dir
      .open_file(&file_name)
      .expect("oracle should find written file");

    let mut actual = Vec::new();
    oracle_file
      .read_to_end(&mut actual)
      .expect("oracle should read written file contents");

    assert_eq!(actual, uuid.as_bytes());
  }

  #[test]
  fn save_to_file_from_xz_source_can_be_verified_by_reading_back_the_raw_fat_image() {
    let uuid = Uuid::new_v4().to_string();
    let file_name = format!("{:08x}.txt", hash(uuid.as_bytes()));

    let xz_fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("tests")
      .join("fixtures")
      .join("test.img.xz");

    let mut image = RpiImage::new(&xz_fixture_path).expect("should open xz fixture image");
    image
      .write_bytes(&file_name, uuid.as_bytes())
      .expect("should write bytes into FAT workspace from xz source");

    let output = NamedTempFile::new().expect("should create output temp file");
    let output_path = output.path().to_path_buf();
    image
      .save_to_file(&output_path)
      .expect("should save patched image from xz source to temp file");

    let mut output_file = File::open(&output_path).expect("should reopen saved image");
    let layout = FatPartitionLayout::new(&mut output_file).expect("should read FAT layout");

    output_file
      .seek(SeekFrom::Start(layout.base))
      .expect("should seek to FAT start");
    let mut fat_bytes = Vec::with_capacity(layout.length as usize);
    (&mut output_file)
      .take(layout.length)
      .read_to_end(&mut fat_bytes)
      .expect("should read extracted FAT bytes");

    let fat_stream = BufStream::new(Cursor::new(fat_bytes));
    let fat_fs = fatfs::FileSystem::new(fat_stream, fatfs::FsOptions::new())
      .expect("oracle should open FAT filesystem from RAM buffer");

    let root_dir = fat_fs.root_dir();
    let mut oracle_file = root_dir
      .open_file(&file_name)
      .expect("oracle should find written file");

    let mut actual = Vec::new();
    oracle_file
      .read_to_end(&mut actual)
      .expect("oracle should read written file contents");

    assert_eq!(actual, uuid.as_bytes());
  }
}
