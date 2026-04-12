use crate::patcher::Instruction;
use crate::patcher::PatchError;
use std::path::PathBuf;

fn extract_arguments(input: &str) -> Vec<String> {
  let mut args = Vec::new();
  let mut current = String::new();
  let mut in_quotes = false;

  for c in input.chars() {
    match c {
      '"' => {
        in_quotes = !in_quotes;
      }
      ' ' if !in_quotes => {
        if !current.is_empty() {
          args.push(current.clone());
          current.clear();
        }
      }
      _ => current.push(c),
    }
  }

  if !current.is_empty() {
    args.push(current);
  }

  args
}

fn validate_instructions(instructions: &[Instruction]) -> Result<(), PatchError> {
  let has_from = instructions
    .iter()
    .any(|i| matches!(i, Instruction::From { .. }));

  if !has_from {
    return Err(PatchError::MissingFromInstruction);
  }

  let has_save = instructions
    .iter()
    .any(|i| matches!(i, Instruction::Save { .. }));
  let has_overwrite = instructions
    .iter()
    .any(|i| matches!(i, Instruction::Overwrite));

  if !(has_save || has_overwrite) {
    return Err(PatchError::MissingSaveOrOverwriteInstruction);
  }
  if has_save && has_overwrite {
    return Err(PatchError::ConflictingSaveInstructions);
  }

  Ok(())
}

pub fn parse_instructions(patcherfile: &str) -> Result<Vec<Instruction>, PatchError> {
  let instructions: Vec<Instruction> = patcherfile
    .lines()
    .filter(|line| !line.is_empty())
    .map(|line| {
      let (cmd, rest) = line.trim().split_once(' ').unwrap_or((line, ""));
      match cmd {
        "FROM" => parse_from(extract_arguments(rest)),
        "EXEC" => parse_exec(extract_arguments(rest)),
        "ADD" => parse_add(extract_arguments(rest)),
        "APPEND" => parse_append(extract_arguments(rest)),
        "SAVE" => parse_save(extract_arguments(rest)),
        "OVERWRITE" => Ok(Instruction::Overwrite),
        other => Err(PatchError::UnknownInstruction(other.to_string())),
      }
    })
    .collect::<Result<Vec<Instruction>, PatchError>>()?;

  validate_instructions(&instructions)?;

  Ok(instructions)
}

fn parse_from(args: Vec<String>) -> Result<Instruction, PatchError> {
  match args.len() {
    0 => Err(PatchError::MissingArgument("FROM".to_owned())),
    1 => Ok(Instruction::From {
      source_image: PathBuf::from(&args[0]),
    }),
    _ => Err(PatchError::InvalidArguments("FROM".to_owned(), args)),
  }
}

fn parse_exec(args: Vec<String>) -> Result<Instruction, PatchError> {
  match args.len() {
    0 => Err(PatchError::MissingArgument("EXEC".to_owned())),
    _ => Ok(Instruction::Exec { args }),
  }
}

fn parse_add(args: Vec<String>) -> Result<Instruction, PatchError> {
  match args.as_slice() {
    [kind, src, dst] if kind == "FILE" => Ok(Instruction::AddFile {
      fat_path: src.clone(),
      host_file: PathBuf::from(dst),
    }),
    _ => Err(PatchError::InvalidArguments("ADD".to_owned(), args)),
  }
}

fn parse_append(args: Vec<String>) -> Result<Instruction, PatchError> {
  match args.as_slice() {
    [kind, src, dst] if kind == "FILE" => Ok(Instruction::AppendFile {
      fat_path: src.clone(),
      host_file: PathBuf::from(dst),
    }),
    _ => Err(PatchError::InvalidArguments("ADD".to_owned(), args)),
  }
}

fn parse_save(args: Vec<String>) -> Result<Instruction, PatchError> {
  match args.len() {
    0 => Err(PatchError::MissingArgument("SAVE".to_owned())),
    _ => Ok(Instruction::Save {
      output_image: PathBuf::from(&args[0]),
    }),
  }
}
