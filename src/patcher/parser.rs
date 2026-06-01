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
  if instructions.is_empty() {
    return Err(PatchError::EmptyPatcherfile);
  }

  let has_from = instructions
    .iter()
    .any(|i| matches!(i, Instruction::From { .. }));
  if !has_from {
    return Err(PatchError::MissingFromInstruction);
  }

  let from_count = instructions
    .iter()
    .filter(|i| matches!(i, Instruction::From { .. }))
    .count();

  if from_count > 1 {
    return Err(PatchError::MultipleFromInstructions);
  }

  let save_count = instructions
    .iter()
    .filter(|i| matches!(i, Instruction::Save { .. }))
    .count();
  if save_count == 0 {
    return Err(PatchError::MissingSaveInstruction);
  }
  if save_count > 1 {
    return Err(PatchError::MultipleSaveInstructions);
  }

  let last = &instructions[instructions.len() - 1];
  if !matches!(last, Instruction::Save { .. }) {
    return Err(PatchError::SaveMustBeLastInstruction);
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
    .map(|line| line.trim())
    .filter(|line| !line.is_empty() && !line.starts_with('#'))
    .map(interpolate_env)
    .map(|line| {
      let (cmd, rest) = line.split_once(' ').unwrap_or((&line, ""));
      match cmd {
        "FROM" => parse_from(extract_arguments(rest)),
        "SHELL" => parse_shell(rest),
        "EXEC" => parse_exec(extract_arguments(rest)),
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

fn parse_shell(command: &str) -> Result<Instruction, PatchError> {
  let command = command.trim();
  match command.is_empty() {
    true => Err(PatchError::MissingArgument("SHELL".to_owned())),
    false => Ok(Instruction::Shell {
      command: command.to_owned(),
    }),
  }
}

fn parse_exec(args: Vec<String>) -> Result<Instruction, PatchError> {
  match args.split_first() {
    None => Err(PatchError::MissingArgument("EXEC".to_owned())),
    Some((program, args)) => Ok(Instruction::Exec {
      program: program.to_owned(),
      args: args.to_vec(),
    }),
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
    _ => Err(PatchError::InvalidArguments("APPEND".to_owned(), args)),
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

#[cfg(test)]
mod tests {
  use super::parse_instructions;
  use crate::patcher::Instruction;
  use crate::patcher::PatchError;

  #[test]
  fn parse_instructions_rejects_empty_patcherfile() {
    assert!(matches!(
      parse_instructions(""),
      Err(PatchError::EmptyPatcherfile)
    ));
  }

  #[test]
  fn parse_instructions_allows_exec_before_from() {
    let patcherfile = r#"
      EXEC echo "prepare"
      FROM "base.img"
      SAVE "out.img"
    "#;

    let instructions = parse_instructions(patcherfile).unwrap();
    assert!(matches!(
      instructions[0],
      Instruction::Exec {
        program: _,
        args: _
      }
    ));
    assert!(matches!(instructions[1], Instruction::From { .. }));
    assert!(matches!(instructions[2], Instruction::Save { .. }));
  }

  #[test]
  fn parse_instructions_parses_exec_program_and_args() {
    let patcherfile = r#"
      EXEC cp "file a.txt" "file b.txt"
      FROM "base.img"
      SAVE "out.img"
    "#;

    let instructions = parse_instructions(patcherfile).unwrap();
    assert!(matches!(
      &instructions[0],
      Instruction::Exec { program, args } if program == "cp" && args == &vec!["file a.txt".to_owned(), "file b.txt".to_owned()]
    ));
  }

  #[test]
  fn parse_instructions_allows_shell_before_from() {
    let patcherfile = r#"
      SHELL echo "prepare" | cat
      FROM "base.img"
      SAVE "out.img"
    "#;

    let instructions = parse_instructions(patcherfile).unwrap();
    assert!(matches!(
      &instructions[0],
      Instruction::Shell { command } if command == "echo \"prepare\" | cat"
    ));
  }

  #[test]
  fn parse_instructions_trims_shell_command() {
    let patcherfile = r#"
      SHELL     echo "prepare"
      FROM "base.img"
      SAVE "out.img"
    "#;

    let instructions = parse_instructions(patcherfile).unwrap();
    assert!(matches!(
      &instructions[0],
      Instruction::Shell { command } if command == "echo \"prepare\""
    ));
  }

  #[test]
  fn parse_instructions_ignores_comments_and_blank_lines() {
    let patcherfile = r#"
      # comment 1

      FROM "base.img"
      # comment 2
      SAVE "out.img"
    "#;

    let instructions = parse_instructions(patcherfile).unwrap();
    assert_eq!(instructions.len(), 2);
    assert!(matches!(instructions[0], Instruction::From { .. }));
    assert!(matches!(instructions[1], Instruction::Save { .. }));
  }

  #[test]
  fn parse_instructions_rejects_multiple_save_instructions() {
    let patcherfile = r#"
      FROM "base.img"
      SAVE "out-a.img"
      SAVE "out-b.img"
    "#;

    assert!(matches!(
      parse_instructions(patcherfile),
      Err(PatchError::MultipleSaveInstructions)
    ));
  }

  #[test]
  fn parse_instructions_rejects_save_not_last() {
    let patcherfile = r#"
      FROM "base.img"
      SAVE "out.img"
      SHELL echo "should fail"
    "#;

    assert!(matches!(
      parse_instructions(patcherfile),
      Err(PatchError::SaveMustBeLastInstruction)
    ));
  }
}
