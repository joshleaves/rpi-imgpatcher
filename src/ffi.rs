use crate::RpiImage;
#[cfg(feature = "ffi_debug")]
use crate::ffi_debug::set_last_error_message;
mod progress_reader;
mod progress_writer;
use crate::rpi_image::Error;
use libc;
use progress_reader::ProgressReader;
use progress_writer::ProgressWriter;
use std::ffi::c_void;
use std::ffi::{CStr, OsStr, c_char};
use std::fs::{File, OpenOptions};
use std::io::{SeekFrom, prelude::*};
use std::os::fd::FromRawFd;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

type ProgressCallback = Option<extern "C" fn(u64, *const c_void)>;

macro_rules! check_not_null {
  ($ptr:expr) => {
    if $ptr.is_null() {
      let err = Error::NullPointer;
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(err.to_string());
      return err.ffi() as i64;
    }
  };
}

macro_rules! check_not_null_out {
  ($out:expr, $ptr:expr) => {
    if $ptr.is_null() {
      let err = Error::NullPointer;
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(err.to_string());
      if !$out.is_null() {
        unsafe { *$out = err.ffi() as u32 };
      }
      return -1;
    }
  };
}

macro_rules! return_error {
  ($err:expr) => {{
    let err: Error = $err.into();
    #[cfg(feature = "ffi_debug")]
    set_last_error_message(err.to_string());
    return err.ffi() as i64;
  }};
}

macro_rules! return_error_out {
  ($out:expr, $err:expr) => {{
    let err: Error = $err.into();
    #[cfg(feature = "ffi_debug")]
    set_last_error_message(err.to_string());
    if !$out.is_null() {
      unsafe { *$out = err.ffi() as u32 };
    }
    return -1;
  }};
}

macro_rules! return_success_out {
  ($out:expr, $ret:expr) => {{
    if !$out.is_null() {
      unsafe { *$out = 0 };
    }
    return $ret;
  }};
}

fn c_char_to_string(string: *const c_char) -> Option<String> {
  if string.is_null() {
    return None;
  }

  let c_str = unsafe { CStr::from_ptr(string) };
  let string = c_str.to_str().ok()?.to_owned();
  Some(string)
}

fn c_char_to_pathbuf(path: *const c_char) -> Option<PathBuf> {
  if path.is_null() {
    return None;
  }

  let c_str = unsafe { CStr::from_ptr(path) };
  let os_str = OsStr::from_bytes(c_str.to_bytes());
  Some(PathBuf::from(os_str))
}

#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_new(image_path: *const c_char) -> *mut RpiImage {
  let Some(image_path) = c_char_to_pathbuf(image_path) else {
    #[cfg(feature = "ffi_debug")]
    set_last_error_message(Error::InvalidArgument.to_string());
    return std::ptr::null_mut();
  };

  let rpi_image = match RpiImage::new(image_path) {
    Err(_err) => {
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(_err.to_string());
      return std::ptr::null_mut();
    }
    Ok(v) => v,
  };

  Box::into_raw(Box::new(rpi_image))
}

// pub extern "C" fn rpi_image_read_file(
//   rpi_image: *mut RpiImage,
//   path: *const c_char
// ) -> {

// }

// pub fn write_file(&mut self, fat_path: &str, file: impl AsRef<Path>) -> Result<u64, Error> {
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_write_file(
  rpi_image: *mut RpiImage,
  fat_path: *const c_char,
  file: *const c_char,
  out_error: *mut u32,
) -> i64 {
  check_not_null_out!(out_error, rpi_image);

  let Some(fat_path) = c_char_to_string(fat_path) else {
    return_error_out!(out_error, Error::InvalidArgument);
  };
  let Some(file) = c_char_to_pathbuf(file) else {
    return_error_out!(out_error, Error::InvalidArgument);
  };
  let rpi_image = unsafe { &mut *rpi_image };
  match rpi_image.write_file(&fat_path, file) {
    Ok(written) => {
      return_success_out!(out_error, written as i64);
    }
    Err(err) => {
      return_error_out!(out_error, err);
    }
  }
}

// pub fn write_bytes(&mut self, fat_path: &str, bytes: &[u8]) -> Result<u64, Error>
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_write_string(
  rpi_image: *mut RpiImage,
  fat_path: *const c_char,
  string: *const c_char,
  out_error: *mut u32,
) -> i64 {
  check_not_null_out!(out_error, rpi_image);

  let Some(fat_path) = c_char_to_string(fat_path) else {
    return_error_out!(out_error, Error::InvalidArgument);
  };
  let Some(string) = c_char_to_string(string) else {
    return_error_out!(out_error, Error::InvalidArgument);
  };
  let rpi_image = unsafe { &mut *rpi_image };
  match rpi_image.write_bytes(&fat_path, string.as_bytes()) {
    Ok(written) => {
      return_success_out!(out_error, written as i64);
    }
    Err(err) => {
      return_error_out!(out_error, err);
    }
  }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_write_bytes(
  rpi_image: *mut RpiImage,
  fat_path: *const c_char,
  bytes_ptr: *const u8,
  bytes_len: usize,
  out_error: *mut u32,
) -> i64 {
  check_not_null_out!(out_error, rpi_image);
  check_not_null_out!(out_error, bytes_ptr);

  let Some(fat_path) = c_char_to_string(fat_path) else {
    return_error_out!(out_error, Error::InvalidArgument);
  };
  let bytes = unsafe { std::slice::from_raw_parts(bytes_ptr, bytes_len) };
  let rpi_image = unsafe { &mut *rpi_image };
  match rpi_image.write_bytes(&fat_path, bytes) {
    Ok(written) => {
      return_success_out!(out_error, written as i64);
    }
    Err(err) => {
      return_error_out!(out_error, err);
    }
  }
}

//pub fn append_bytes(&mut self, fat_path: &str, bytes: &[u8]) -> Result<u64, Error>
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_append_string(
  rpi_image: *mut RpiImage,
  fat_path: *const c_char,
  string: *const c_char,
  out_error: *mut u32,
) -> i64 {
  check_not_null_out!(out_error, rpi_image);

  let Some(fat_path) = c_char_to_string(fat_path) else {
    return_error_out!(out_error, Error::InvalidArgument);
  };
  let Some(string) = c_char_to_string(string) else {
    return_error_out!(out_error, Error::InvalidArgument);
  };
  let rpi_image = unsafe { &mut *rpi_image };
  match rpi_image.append_bytes(&fat_path, string.as_bytes()) {
    Err(err) => return_error_out!(out_error, err),
    Ok(written) => return_success_out!(out_error, written as i64),
  }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_append_bytes(
  rpi_image: *mut RpiImage,
  fat_path: *const c_char,
  bytes_ptr: *const u8,
  bytes_len: usize,
  out_error: *mut u32,
) -> i64 {
  check_not_null_out!(out_error, rpi_image);
  if bytes_ptr.is_null() {
    return_error_out!(out_error, Error::NullPointer);
  }

  let Some(fat_path) = c_char_to_string(fat_path) else {
    return_error_out!(out_error, Error::InvalidArgument);
  };
  let bytes = unsafe { std::slice::from_raw_parts(bytes_ptr, bytes_len) };
  let rpi_image = unsafe { &mut *rpi_image };
  match rpi_image.append_bytes(&fat_path, bytes) {
    Err(err) => return_error_out!(out_error, err),
    Ok(written) => return_success_out!(out_error, written as i64),
  }
}

// pub fn save_to_file(self, file: impl AsRef<Path>) -> Result<(), Error>
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_save_to_file(rpi_image: *mut RpiImage, file: *const c_char) -> i64 {
  check_not_null!(rpi_image);

  let Some(file) = c_char_to_pathbuf(file) else {
    return_error!(Error::InvalidArgument);
  };
  let rpi_image = unsafe { &mut *rpi_image };

  match rpi_image.save_to_file(file) {
    Err(err) => return_error!(err),
    Ok(_) => 0,
  }
}

// pub fn save_to_file(self, file: impl AsRef<Path>) -> Result<(), Error>
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_save_to_file_with_progress(
  rpi_image: *mut RpiImage,
  file: *const c_char,
  progress: ProgressCallback,
  context: *const c_void,
) -> i64 {
  check_not_null!(rpi_image);
  let Some(progress) = progress else {
    return_error!(Error::NullPointer);
  };

  let Some(file) = c_char_to_pathbuf(file) else {
    return_error!(Error::InvalidArgument)
  };
  let rpi_image = unsafe { &mut *rpi_image };

  let file = OpenOptions::new()
    .create(true)
    .truncate(true)
    .read(true)
    .write(true)
    .open(file);
  let file = match file {
    Err(err) => return_error!(err),
    Ok(f) => f,
  };

  let cb = |written| progress(written, context);
  let mut writer = ProgressWriter::new(file, cb);
  match rpi_image.save_to_writer(&mut writer) {
    Err(err) => return_error!(err),
    Ok(_) => 0,
  }
}

// pub(crate) fn save_to_fd(self, fd: RawFd) -> Result<(), Error>
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_save_to_fd(rpi_image: *mut RpiImage, fd: i32) -> i64 {
  check_not_null!(rpi_image);

  let rpi_image = unsafe { &mut *rpi_image };
  let file = unsafe { libc::dup(fd) };
  if file < 0 {
    return_error!(Error::CannotDuplicateFD);
  }
  let mut file = unsafe { File::from_raw_fd(file) };

  match rpi_image.save_to_writer(&mut file) {
    Err(err) => return_error!(err),
    Ok(_) => 0,
  }
}

// pub(crate) fn save_to_fd_with_progress<F>(self, fd: RawFd, progress: FnMut(u64)) -> Result<(), Error>
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_save_to_fd_with_progress(
  rpi_image: *mut RpiImage,
  fd: i32,
  progress: ProgressCallback,
  context: *const c_void,
) -> i64 {
  check_not_null!(rpi_image);
  let Some(progress) = progress else {
    return_error!(Error::NullPointer)
  };

  let rpi_image = unsafe { &mut *rpi_image };
  let file = unsafe { libc::dup(fd) };
  if file < 0 {
    return_error!(Error::CannotDuplicateFD);
  }
  let file = unsafe { File::from_raw_fd(file) };
  let cb = |written| progress(written, context);
  let mut writer = ProgressWriter::new(file, cb);

  match rpi_image.save_to_writer(&mut writer) {
    Err(err) => return_error!(err),
    Ok(_) => 0,
  }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_verify_file(rpi_image: *mut RpiImage, file: *const c_char) -> i64 {
  check_not_null!(rpi_image);

  let Some(file) = c_char_to_pathbuf(file) else {
    return_error!(Error::InvalidArgument)
  };
  let rpi_image = unsafe { &mut *rpi_image };

  match rpi_image.verify_file(file) {
    Err(err) => return_error!(err),
    Ok(false) => return_error!(Error::CopyMismatch),
    Ok(true) => 0,
  }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_verify_file_with_progress(
  rpi_image: *mut RpiImage,
  file: *const c_char,
  progress: ProgressCallback,
  context: *const c_void,
) -> i64 {
  check_not_null!(rpi_image);
  let Some(progress) = progress else {
    return_error!(Error::NullPointer)
  };

  let Some(file) = c_char_to_pathbuf(file) else {
    return_error!(Error::InvalidArgument)
  };
  let rpi_image = unsafe { &mut *rpi_image };
  let file = match File::open(file) {
    Err(err) => return_error!(err),
    Ok(f) => f,
  };
  let cb = |written| progress(written, context);
  let mut reader = ProgressReader::new(file, cb);

  match rpi_image.verify_reader(&mut reader) {
    Err(err) => return_error!(err),
    Ok(false) => return_error!(Error::CopyMismatch),
    Ok(true) => 0,
  }
}

// pub(crate) fn save_to_fd(self, fd: RawFd) -> Result<(), Error>
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_verify_fd(rpi_image: *mut RpiImage, fd: i32) -> i64 {
  check_not_null!(rpi_image);

  let rpi_image = unsafe { &mut *rpi_image };
  let file = unsafe { libc::dup(fd) };
  if file < 0 {
    return_error!(Error::CannotDuplicateFD);
  }
  let mut file = unsafe { File::from_raw_fd(file) };
  if let Err(err) = file.seek(SeekFrom::Start(0)) {
    return_error!(err);
  }

  match rpi_image.verify_reader(&mut file) {
    Err(err) => return_error!(err),
    Ok(false) => return_error!(Error::CopyMismatch),
    Ok(true) => 0,
  }
}

// pub(crate) fn verify_fd_with_progress<F>(self, fd: RawFd, progress: FnMut(u64)) -> Result<(), Error>
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_verify_fd_with_progress(
  rpi_image: *mut RpiImage,
  fd: i32,
  progress: ProgressCallback,
  context: *const c_void,
) -> i64 {
  check_not_null!(rpi_image);
  let Some(progress) = progress else {
    return_error!(Error::NullPointer)
  };

  let rpi_image = unsafe { &mut *rpi_image };
  let file = unsafe { libc::dup(fd) };
  if file < 0 {
    return_error!(Error::CannotDuplicateFD);
  }
  let mut file = unsafe { File::from_raw_fd(file) };
  if let Err(err) = file.seek(SeekFrom::Start(0)) {
    return_error!(err);
  }
  let cb = |read| progress(read, context);
  let mut reader = ProgressReader::new(file, cb);

  match rpi_image.verify_reader(&mut reader) {
    Err(err) => return_error!(err),
    Ok(false) => return_error!(Error::CopyMismatch),
    Ok(true) => 0,
  }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_free(rpi_image: *mut RpiImage) {
  if rpi_image.is_null() {
    return;
  };

  unsafe {
    std::mem::drop(Box::from_raw(rpi_image));
  }
}
