use tempfile::PersistError;

#[repr(C)]
#[derive(Debug)]
pub enum Error {
  CopyMismatch,
  InvalidImage,
  InvalidArgument,
  InvalidState,
  Io,

  Fs,
  Internal,
}

impl From<mbrman::Error> for Error {
  fn from(_: mbrman::Error) -> Self {
    Error::Io
  }
}

impl From<PersistError> for Error {
  fn from(_: PersistError) -> Self {
    Error::Io
  }
}

impl From<std::io::Error> for Error {
  fn from(_: std::io::Error) -> Self {
    Error::Io
  }
}
