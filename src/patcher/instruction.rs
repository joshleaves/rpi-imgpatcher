use rpi_imgpatcher::RpiImage;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::patcher::PatchContext;
use crate::patcher::PatchError;

pub enum Instruction {
  From {
    source_image: PathBuf,
  },
  AddFile {
    fat_path: String,
    host_file: PathBuf,
  },
  AppendFile {
    fat_path: String,
    host_file: PathBuf,
  },
  Save {
    output_image: PathBuf,
  },
  Overwrite,
  Exec {
    args: Vec<String>,
  },
}

impl Instruction {
  pub fn execute(&self, ctx: &mut PatchContext) -> Result<(), PatchError> {
    match self {
      Instruction::From { source_image } => self.execute_from(ctx, source_image),
      Instruction::Exec { args } => self.execute_exec(ctx, args),
      Instruction::AddFile {
        fat_path,
        host_file,
      } => self.execute_add_file(ctx, fat_path, host_file),
      Instruction::AppendFile {
        fat_path,
        host_file,
      } => self.execute_append_file(ctx, fat_path, host_file),
      Instruction::Save { output_image } => self.execute_save(ctx, output_image),
      Instruction::Overwrite => self.execute_overwrite(ctx),
    }
  }

  fn execute_from(&self, ctx: &mut PatchContext, source_image: &PathBuf) -> Result<(), PatchError> {
    if ctx.has_image() {
      return Err(PatchError::MultipleFromInstructions);
    }
    let Ok(rpi_image) = RpiImage::new(source_image) else {
      return Err(PatchError::CouldNotInitializeSourceImage(
        source_image.to_path_buf(),
      ));
    };
    ctx.rpi_image = Some(rpi_image);
    Ok(())
  }

  fn execute_exec(&self, _ctx: &mut PatchContext, args: &[String]) -> Result<(), PatchError> {
    if args.is_empty() {
      return Err(PatchError::MissingArgument("EXEC".to_string()));
    }

    let mut cmd = Command::new(&args[0]);

    if args.len() > 1 {
      cmd.args(&args[1..]);
    }

    let status = cmd
      .status()
      .map_err(|_| PatchError::ExecFailed(-1, args.to_owned()))?;

    if !status.success() {
      let code = status.code().unwrap_or(-1);
      return Err(PatchError::ExecFailed(code, args.to_owned()));
    }

    Ok(())
  }

  fn execute_add_file(
    &self,
    ctx: &mut PatchContext,
    fat_path: &String,
    host_file: &PathBuf,
  ) -> Result<(), PatchError> {
    let Some(rpi_image) = &mut ctx.rpi_image else {
      return Err(PatchError::CannotAddFileBeforeFromInstruction(
        fat_path.to_owned(),
      ));
    };
    let bytes =
      fs::read(host_file).map_err(|err| PatchError::CannotReadHostFile(host_file.clone(), err))?;
    rpi_image
      .write_bytes(fat_path, &bytes)
      .map_err(|err| PatchError::CouldNotWriteToFat(fat_path.to_owned(), err))?;

    Ok(())
  }

  fn execute_append_file(
    &self,
    ctx: &mut PatchContext,
    fat_path: &String,
    host_file: &PathBuf,
  ) -> Result<(), PatchError> {
    let Some(rpi_image) = &mut ctx.rpi_image else {
      return Err(PatchError::CannotAppendFileBeforeFromInstruction(
        fat_path.to_owned(),
      ));
    };
    let bytes =
      fs::read(host_file).map_err(|err| PatchError::CannotReadHostFile(host_file.clone(), err))?;
    rpi_image
      .append_bytes(fat_path, &bytes)
      .map_err(|err| PatchError::CouldNotWriteToFat(fat_path.to_owned(), err))?;

    Ok(())
  }

  fn execute_save(&self, ctx: &mut PatchContext, output_image: &PathBuf) -> Result<(), PatchError> {
    let Some(rpi_image) = ctx.rpi_image.take() else {
      return Err(PatchError::CannotSaveBeforeFromInstruction(
        output_image.clone(),
      ));
    };
    rpi_image
      .save_to_file(output_image)
      .map_err(|err| PatchError::CouldNotSaveImage(output_image.clone(), err))?;

    Ok(())
  }

  fn execute_overwrite(&self, ctx: &mut PatchContext) -> Result<(), PatchError> {
    let Some(rpi_image) = ctx.rpi_image.take() else {
      return Err(PatchError::CannotOverwriteBeforeFromInstruction);
    };
    rpi_image
      .overwrite_in_place()
      .map_err(PatchError::CouldNotOverwriteImage)?;

    Ok(())
  }
}
