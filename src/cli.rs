use rpi_imgpatcher::rpi_image::progress_writer::ProgressWriter;
use std::fs;
use std::io::Write;
use std::path::Path;
mod patcher;

use crate::patcher::PatchContext;
use crate::patcher::instruction::Instruction;
use crate::patcher::parser::parse_instructions;

const PROGRAM: &str = "rpi-imgpatcher";

macro_rules! error_exit {
  ($fmt:expr $(, $arg:tt)*) => {{
    eprintln!(concat!("{}: ", $fmt), PROGRAM $(, $arg)*);
    std::process::exit(1);
  }};
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let patcherfile_path = if args.len() > 1 {
    &args[1]
  } else {
    "./Patcherfile"
  };

  if !Path::new(patcherfile_path).exists() {
    error_exit!("Missing Patcherfile at {}", patcherfile_path);
  }
  let patcherfile = match fs::read_to_string(patcherfile_path) {
    Ok(f) => f,
    Err(err) => error_exit!("Could not read Patcherfile {} ({})", patcherfile_path, err),
  };
  let instructions: Vec<Instruction> = match parse_instructions(&patcherfile) {
    Err(err) => error_exit!("{}", err),
    Ok(instructions) => instructions,
  };

  let mut patch_ctx = PatchContext::new();
  for instr in instructions {
    match instr {
      Instruction::Save { output_image } => {
        let rpi_image = patch_ctx
          .rpi_image
          .as_mut()
          .expect("SAVE instruction without image");
        let total_size = rpi_image.layout.length;

        let mut last_percent = 0;
        let progress = move |written: u64| {
          let percent = (written * 100 / total_size).min(100);
          if percent > last_percent {
            print!("\rSaving: {}%", percent);
            std::io::stdout().flush().unwrap();
            last_percent = percent;
          }
        };

        let file = fs::OpenOptions::new()
          .create(true)
          .truncate(true)
          .write(true)
          .open(&output_image);

        let file = match file {
          Ok(f) => f,
          Err(err) => error_exit!("Could not open output image {:?} ({})", output_image, err),
        };

        let mut writer = ProgressWriter::new(file, progress);
        if let Err(err) = rpi_image.save_to_writer(&mut writer) {
          error_exit!("Could not save image: {:?} ({})", output_image, err);
        }
        println!("\rSaved to {:?}", output_image);
      }
      _ => {
        if let Err(err) = instr.execute(&mut patch_ctx) {
          error_exit!("{}", err);
        }
      }
    }
  }

  std::process::exit(0);
}
