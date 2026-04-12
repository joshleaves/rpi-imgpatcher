use crate::patcher::PatchContext;
use crate::patcher::PatchError;
use rpi_imgpatcher::RpiImage;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

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
  Exec {
    command: String,
  },
}

impl Instruction {
  pub fn execute(&self, ctx: &mut PatchContext) -> Result<(), PatchError> {
    match self {
      Instruction::From { source_image } => self.execute_from(ctx, source_image),
      Instruction::Exec { command } => self.execute_exec(ctx, command),
      Instruction::AddFile {
        fat_path,
        host_file,
      } => self.execute_add_file(ctx, fat_path, host_file),
      Instruction::AppendFile {
        fat_path,
        host_file,
      } => self.execute_append_file(ctx, fat_path, host_file),
      Instruction::Save { output_image } => self.execute_save(ctx, output_image),
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

  fn execute_exec(&self, _ctx: &mut PatchContext, command: &String) -> Result<(), PatchError> {
    // println!("COMMAND RAW: {:?}", command);
    let status = Command::new("sh")
      .stderr(Stdio::null())
      .arg("-o")
      .arg("pipefail")
      .arg("-c")
      .arg(command)
      .status()
      .map_err(|_| PatchError::ExecFailed(-1, command.to_owned()))?;

    // println!("STATUS: {:?}", status);
    if !status.success() {
      let code = status.code().unwrap_or(-1);
      return Err(PatchError::ExecFailed(code, command.to_owned()));
    }
    // println!("STATUS: {:?} : {:?}", status.success(), status.code());

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
}
