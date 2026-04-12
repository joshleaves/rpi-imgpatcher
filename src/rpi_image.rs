use fatfs::FileSystem;
use fscommon::BufStream;
use std::fs::File;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
mod layout;
pub use layout::FatPartitionLayout;
mod error;
pub use error::Error;
mod fat_file;
mod image_io;
mod save;
use save::SaveStrategy;

pub struct RpiImage {
  // Path and handle to the original disk image.
  // The original image is never modified until an explicit save step.
  image_path: PathBuf,

  // Byte range of the boot FAT partition inside the source image.
  fat_base: u64,
  fat_length: u64,

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
    let mut image_file = File::open(&image_path)?;

    let fat_tmp = NamedTempFile::new()?;
    let (mut fat_tmp_file, fat_tmp_path) = fat_tmp.keep()?;

    let layout_fat = FatPartitionLayout::new(&mut image_file)?;
    image_io::extract_fat_to_file(
      &mut image_file,
      &mut fat_tmp_file,
      layout_fat.base,
      layout_fat.length,
    )?;
    let buf_stream = BufStream::new(fat_tmp_file);
    let fat = fatfs::FileSystem::new(buf_stream, fatfs::FsOptions::new())?;

    Ok(Self {
      image_path,

      fat_base: layout_fat.base,
      fat_length: layout_fat.length,

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
  pub fn save_to_file(self, file: impl AsRef<Path>) -> Result<(), Error> {
    SaveStrategy::ToFile(file.as_ref().to_path_buf()).save(self)
  }

  /// Write the modified FAT workspace back into the original image file.
  ///
  /// This mutates the source image in place and should be treated as a
  /// destructive operation.
  pub fn overwrite_in_place(self) -> Result<(), Error> {
    SaveStrategy::Overwrite.save(self)
  }
}
