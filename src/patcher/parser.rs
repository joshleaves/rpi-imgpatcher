use crate::patcher::Instruction;
use crate::patcher::PatchError;
use std::env;
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

  if !has_save {
    return Err(PatchError::MissingSaveInstruction);
  }

  Ok(())
}

fn interpolate_env(input: &str) -> String {
  let mut result = String::new();
  let mut chars = input.chars().peekable();

  while let Some(c) = chars.next() {
    if c == '$' {
      let mut var = String::new();

      while let Some(&next) = chars.peek() {
        if next.is_alphanumeric() || next == '_' {
          var.push(next);
          chars.next();
        } else {
          break;
        }
      }

      if !var.is_empty() {
        if let Ok(value) = env::var(&var) {
          result.push_str(&value);
        }
        // si var n'existe pas → remplacé par ""
      } else {
        result.push('$');
      }
    } else {
      result.push(c);
    }
  }

  result
}

pub fn parse_instructions(patcherfile: &str) -> Result<Vec<Instruction>, PatchError> {
  let instructions: Vec<Instruction> = patcherfile
    .lines()
    .filter(|line| !line.is_empty())
    .map(interpolate_env)
    .map(|line| {
      let (cmd, rest) = line.trim().split_once(' ').unwrap_or((&line, ""));
      match cmd {
        "FROM" => parse_from(extract_arguments(rest)),
        "EXEC" => parse_exec(rest.to_string()),
        "ADD" => parse_add(extract_arguments(rest)),
        "APPEND" => parse_append(extract_arguments(rest)),
        "SAVE" => parse_save(extract_arguments(rest)),
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

fn parse_exec(command: String) -> Result<Instruction, PatchError> {
  match command.is_empty() {
    true => Err(PatchError::MissingArgument("EXEC".to_owned())),
    false => Ok(Instruction::Exec { command }),
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
    [kind, append_conf] if kind == "CMDLINE" => Ok(Instruction::AppendCmdline {
      append_conf: append_conf.clone(),
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
