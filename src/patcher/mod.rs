use std::path::PathBuf;

use std::fmt::{self};

use rpi_imgpatcher::{RpiImage, rpi_image};

pub mod instruction;
pub use instruction::Instruction;
pub mod parser;

pub struct PatchContext {
  rpi_image: Option<RpiImage>,
}

impl PatchContext {
  pub fn new() -> Self {
    Self { rpi_image: None }
  }

  pub fn has_image(&self) -> bool {
    self.rpi_image.is_some()
  }
}

pub enum PatchError {
  UnknownInstruction(String),
  MissingArgument(String),
  InvalidArguments(String, Vec<String>),
  MissingFromInstruction,
  MissingSaveInstruction,
  MultipleFromInstructions,
  CouldNotInitializeSourceImage(PathBuf),
  ExecFailed(i32, String),
  CannotAddFileBeforeFromInstruction(String),
  CannotAppendFileBeforeFromInstruction(String),
  CannotAppendToCmdlineBeforeFromInstruction,
  CannotReadCmdlineTxt,
  CannotAppendtoCmdlineTxt,
  CannotReadHostFile(PathBuf, std::io::Error),
  CouldNotWriteToFat(String, rpi_image::Error),
  CannotSaveBeforeFromInstruction(PathBuf),
  CouldNotSaveImage(PathBuf, rpi_image::Error),
}

impl fmt::Display for PatchError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      PatchError::UnknownInstruction(instruction) => {
        write!(f, "Unknown instruction: {}", instruction)
      }
      PatchError::MissingArgument(instruction) => {
        write!(f, "Instruction {} is missing an argument", instruction)
      }
      PatchError::InvalidArguments(instruction, args) => {
        write!(
          f,
          "Instruction {} has invalid arguments: {:?}",
          instruction, args
        )
      }
      PatchError::MissingFromInstruction => {
        write!(f, "Missing FROM instruction")
      }
      PatchError::MissingSaveInstruction => {
        write!(f, "Missing SAVE instruction")
      }
      PatchError::CouldNotInitializeSourceImage(source_image) => {
        write!(f, "Could not initialize source image: {:?}", source_image)
      }
      PatchError::MultipleFromInstructions => {
        write!(f, "Multiple FROM instructions")
      }
      PatchError::ExecFailed(status, command) => {
        write!(f, "Command failed ({}): `{}`", status, command)
      }
      PatchError::CannotAddFileBeforeFromInstruction(fat_path) => {
        write!(
          f,
          "Cannot use ADD FILE before FROM instruction: {}",
          fat_path
        )
      }
      PatchError::CannotAppendFileBeforeFromInstruction(fat_path) => {
        write!(
          f,
          "Cannot use ADD FILE before FROM instruction: {}",
          fat_path
        )
      }
      PatchError::CannotAppendToCmdlineBeforeFromInstruction => {
        write!(f, "Cannot APPEND CMDLINE before FROM instruction")
      }
      PatchError::CannotReadCmdlineTxt => {
        write!(f, "Cannot read `cmdline.txt`")
      }
      PatchError::CannotAppendtoCmdlineTxt => {
        write!(f, "Error writing `cmdline.txt`")
      }
      PatchError::CannotReadHostFile(host_file, io_error) => {
        write!(f, "Cannot read host file {:?}: {}", host_file, io_error)
      }
      PatchError::CouldNotWriteToFat(fat_path, fat_error) => {
        write!(
          f,
          "Could not write file to FAT: {} ({:?})",
          fat_path, fat_error
        )
      }
      PatchError::CannotSaveBeforeFromInstruction(output_image) => {
        write!(
          f,
          "Cannot use SAVE before FROM instruction: {:?}",
          output_image
        )
      }
      PatchError::CouldNotSaveImage(output_image, err) => {
        write!(f, "Could not save image: {:?} ({:?})", output_image, err)
      }
    }
  }
}
