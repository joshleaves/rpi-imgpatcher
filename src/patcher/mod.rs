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

#[derive(Debug)]
pub enum PatchError {
  EmptyPatcherfile,
  UnknownInstruction(String),
  MissingArgument(String),
  InvalidArguments(String, Vec<String>),
  MissingFromInstruction,
  MissingSaveInstruction,
  MultipleFromInstructions,
  MultipleSaveInstructions,
  SaveMustBeLastInstruction,
  CouldNotInitializeSourceImage(PathBuf, rpi_image::Error),
  ShellFailed(i32, String),
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
      PatchError::EmptyPatcherfile => {
        write!(f, "Patcherfile is empty")
      }
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
      PatchError::CouldNotInitializeSourceImage(source_image, err) => {
        write!(
          f,
          "Could not initialize source image: {:?} ({:?})",
          source_image, err
        )
      }
      PatchError::MultipleFromInstructions => {
        write!(f, "Multiple FROM instructions")
      }
      PatchError::MultipleSaveInstructions => {
        write!(f, "Multiple SAVE instructions")
      }
      PatchError::SaveMustBeLastInstruction => {
        write!(f, "SAVE instruction must be the last instruction")
      }
      PatchError::ShellFailed(status, command) => {
        write!(f, "Shell command failed ({}): `{}`", status, command)
      }
      PatchError::ExecFailed(status, command) => {
        write!(f, "EXEC command failed ({}): `{}`", status, command)
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
          "Cannot use APPEND FILE before FROM instruction: {}",
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
