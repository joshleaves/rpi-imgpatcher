use std::cell::Cell;
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
  let last_percent = Cell::new(0_u64);
  let mut progress = |written: u64, total: u64| {
    let percent = if total == 0 {
      100
    } else {
      (written * 100 / total).min(100)
    };
    if percent > last_percent.get() {
      print!("\rSaving: {}%", percent);
      let _ = std::io::stdout().flush();
      last_percent.set(percent);
    }
  };

  for instr in instructions {
    let result = match &instr {
      Instruction::Save { .. } => instr.execute_with_progress(&mut patch_ctx, Some(&mut progress)),
      _ => instr.execute(&mut patch_ctx),
    };

    if let Err(err) = result {
      error_exit!("{}", err);
    }

    if let Instruction::Save { output_image } = &instr {
      println!("\rSaved to {:?}", output_image);
      last_percent.set(0);
    }
  }

  std::process::exit(0);
}
