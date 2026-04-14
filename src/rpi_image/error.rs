use mbrman;
use std::fmt::{self};
use std::io;
use tempfile::PersistError;

#[derive(Debug)]
pub enum Error {
  NullPointer,
  CopyMismatch,
  InvalidImage,
  InvalidArgument,
  InvalidState,
  Io(std::io::Error),
  Mbr(mbrman::Error),
  TempFile(PersistError),
}

#[repr(u32)]
pub enum FfiError {
  NullPointer = 1,
  CopyMismatch = 2,
  InvalidImage = 3,
  InvalidArgument = 4,
  InvalidState = 5,
  Io = 6,
  Mbr = 7,
  TempFile = 8,
}

impl Error {
  pub fn ffi(&self) -> FfiError {
    match self {
      Error::NullPointer => FfiError::NullPointer,
      Error::CopyMismatch => FfiError::CopyMismatch,
      Error::InvalidImage => FfiError::InvalidImage,
      Error::InvalidArgument => FfiError::InvalidArgument,
      Error::InvalidState => FfiError::InvalidState,
      Error::Io(_) => FfiError::Io,
      Error::Mbr(_) => FfiError::Mbr,
      Error::TempFile(_) => FfiError::TempFile,
    }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::NullPointer => write!(f, "Null pointer"),
      Error::CopyMismatch => write!(f, "Copy mismatch"),
      Error::InvalidImage => write!(f, "Invalid image"),
      Error::InvalidArgument => write!(f, "Invalid Argument"),
      Error::InvalidState => write!(f, "Invalid State"),
      Error::Io(e) => write!(f, "I/O Error ({})", e),
      Error::Mbr(e) => write!(f, "MBR Error ({})", e),
      Error::TempFile(e) => write!(f, "Tempfile Error ({})", e),
    }
  }
}

impl From<mbrman::Error> for Error {
  fn from(err: mbrman::Error) -> Self {
    Error::Mbr(err)
  }
}

impl From<PersistError> for Error {
  fn from(err: PersistError) -> Self {
    Error::TempFile(err)
  }
}

impl From<io::Error> for Error {
  fn from(err: io::Error) -> Self {
    Error::Io(err)
  }
}
