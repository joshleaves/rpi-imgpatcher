use std::fs;
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
    match instr.execute(&mut patch_ctx) {
      Ok(_) => (),
      Err(err) => error_exit!("{}", err),
    }
  }

  std::process::exit(0);
}
