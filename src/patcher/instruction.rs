use crate::patcher::PatchContext;
use crate::patcher::PatchError;
use rpi_imgpatcher::RpiImage;
use rpi_imgpatcher::rpi_image::progress_writer::ProgressWriter;
use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::Command;

pub enum Instruction {
  From {
    source_image: PathBuf,
  },
  Shell {
    command: String,
  },
  Exec {
    program: String,
    args: Vec<String>,
  },
  AddFile {
    fat_path: String,
    host_file: PathBuf,
  },
  AppendFile {
    fat_path: String,
    host_file: PathBuf,
  },
  AppendCmdline {
    append_conf: String,
  },
  Save {
    output_image: PathBuf,
  },
}

impl Instruction {
  pub fn execute(&self, ctx: &mut PatchContext) -> Result<(), PatchError> {
    self.execute_with_progress(ctx, None)
  }

  /// Executes an instruction with an optional progress callback.
  ///
  /// The callback receives `(written, total)` and is currently emitted by
  /// `SAVE` instructions.
  pub fn execute_with_progress(
    &self,
    ctx: &mut PatchContext,
    on_progress: Option<&mut dyn FnMut(u64, u64)>,
  ) -> Result<(), PatchError> {
    match self {
      Instruction::From { source_image } => self.execute_from(ctx, source_image),
      Instruction::Shell { command } => self.execute_shell(ctx, command),
      Instruction::Exec { program, args } => self.execute_exec(ctx, program, args),
      Instruction::AddFile {
        fat_path,
        host_file,
      } => self.execute_add_file(ctx, fat_path, host_file),
      Instruction::AppendFile {
        fat_path,
        host_file,
      } => self.execute_append_file(ctx, fat_path, host_file),
      Instruction::AppendCmdline { append_conf } => self.execute_append_cmdline(ctx, append_conf),
      Instruction::Save { output_image } => self.execute_save(ctx, output_image, on_progress),
    }
  }

  fn execute_from(&self, ctx: &mut PatchContext, source_image: &PathBuf) -> Result<(), PatchError> {
    if ctx.has_image() {
      return Err(PatchError::MultipleFromInstructions);
    }
    let rpi_image = RpiImage::new(source_image)
      .map_err(|err| PatchError::CouldNotInitializeSourceImage(source_image.to_path_buf(), err))?;
    ctx.rpi_image = Some(rpi_image);
    Ok(())
  }

  fn execute_shell(&self, _ctx: &mut PatchContext, command: &String) -> Result<(), PatchError> {
    let status = Command::new("/bin/sh")
      .arg("-o")
      .arg("pipefail")
      .arg("-c")
      .arg(command)
      .status()
      .map_err(|_| PatchError::ShellFailed(-1, command.to_owned()))?;

    if !status.success() {
      let code = status.code().unwrap_or(-1);
      return Err(PatchError::ShellFailed(code, command.to_owned()));
    }

    Ok(())
  }

  fn execute_exec(
    &self,
    _ctx: &mut PatchContext,
    program: &String,
    args: &[String],
  ) -> Result<(), PatchError> {
    let command_display = std::iter::once(program.as_str())
      .chain(args.iter().map(String::as_str))
      .collect::<Vec<_>>()
      .join(" ");

    let status = Command::new(program)
      .args(args)
      .status()
      .map_err(|_| PatchError::ExecFailed(-1, command_display.clone()))?;

    if !status.success() {
      let code = status.code().unwrap_or(-1);
      return Err(PatchError::ExecFailed(code, command_display));
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

  fn execute_append_cmdline(
    &self,
    ctx: &mut PatchContext,
    append_conf: &String,
  ) -> Result<(), PatchError> {
    let Some(rpi_image) = &mut ctx.rpi_image else {
      return Err(PatchError::CannotAppendToCmdlineBeforeFromInstruction);
    };
    let Ok(mut buf) = rpi_image.read_file("cmdline.txt") else {
      return Err(PatchError::CannotReadCmdlineTxt);
    };
    while matches!(buf.last(), Some(b'\n' | b'\r' | b' ')) {
      buf.pop();
    }
    if !buf.is_empty() {
      buf.push(b' ');
    }
    buf.extend_from_slice(append_conf.as_bytes());
    rpi_image
      .write_bytes("cmdline.txt", &buf)
      .map_err(|_| PatchError::CannotAppendtoCmdlineTxt)?;

    Ok(())
  }

  fn execute_save(
    &self,
    ctx: &mut PatchContext,
    output_image: &PathBuf,
    on_progress: Option<&mut dyn FnMut(u64, u64)>,
  ) -> Result<(), PatchError> {
    let Some(rpi_image) = &mut ctx.rpi_image else {
      return Err(PatchError::CannotSaveBeforeFromInstruction(
        output_image.clone(),
      ));
    };

    match on_progress {
      Some(progress) => {
        let total_size = rpi_image.fat_length();
        let file = OpenOptions::new()
          .create(true)
          .truncate(true)
          .write(true)
          .open(output_image)
          .map_err(|err| PatchError::CouldNotSaveImage(output_image.clone(), err.into()))?;

        let mut writer = ProgressWriter::new(file, |written| progress(written, total_size));
        rpi_image
          .save_to_writer(&mut writer)
          .map_err(|err| PatchError::CouldNotSaveImage(output_image.clone(), err))?;
      }
      None => {
        rpi_image
          .save_to_file(output_image)
          .map_err(|err| PatchError::CouldNotSaveImage(output_image.clone(), err))?;
      }
    }

    Ok(())
  }
}
