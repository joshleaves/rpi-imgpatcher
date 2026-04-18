use crate::RpiImage;
#[cfg(feature = "ffi_debug")]
use crate::ffi_debug::set_last_error_message;
use crate::rpi_image::Error;
use crate::rpi_image::progress_reader::ProgressReader;
use crate::rpi_image::progress_writer::ProgressWriter;
use std::ffi::c_void;
use std::ffi::{CStr, OsStr, c_char};
use std::fs::{File, OpenOptions};
use std::os::fd::FromRawFd;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

type ProgressCallback = Option<extern "C" fn(u64, *const c_void)>;

macro_rules! check_not_null {
  ($ptr:expr, $ret:expr) => {
    if $ptr.is_null() {
      return $ret;
    }
  };
}

macro_rules! return_out {
  ($out:expr, $err:expr, $type:ty) => {{
    #[cfg(feature = "ffi_debug")]
    set_last_error_message($err.to_string());
    if !$out.is_null() {
      unsafe { *$out = $err.ffi() as $type };
    }
    return -1;
  }};
}

macro_rules! success_out {
  ($out:expr, $ret:expr) => {{
    if !$out.is_null() {
      unsafe { *$out = 0 };
    }
    return $ret;
  }};
}

fn c_char_to_string(string: *const c_char) -> Option<String> {
  check_not_null!(string, None);

  let c_str = unsafe { CStr::from_ptr(string) };
  let string = c_str.to_str().ok()?.to_owned();
  Some(string)
}

fn c_char_to_pathbuf(path: *const c_char) -> Option<PathBuf> {
  check_not_null!(path, None);

  let c_str = unsafe { CStr::from_ptr(path) };
  let os_str = OsStr::from_bytes(c_str.to_bytes());
  Some(PathBuf::from(os_str))
}

#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_new(image_path: *const c_char) -> *mut RpiImage {
  let Some(image_path) = c_char_to_pathbuf(image_path) else {
    return std::ptr::null_mut();
  };

  let Ok(rpi_image) = RpiImage::new(image_path) else {
    return std::ptr::null_mut();
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
  if rpi_image.is_null() {
    return_out!(out_error, Error::NullPointer, u32);
  }

  let Some(fat_path) = c_char_to_string(fat_path) else {
    return_out!(out_error, Error::InvalidArgument, u32);
  };
  let Some(file) = c_char_to_pathbuf(file) else {
    return_out!(out_error, Error::InvalidArgument, u32);
  };
  let rpi_image = unsafe { &mut *rpi_image };
  match rpi_image.write_file(&fat_path, file) {
    Ok(bytes_written) => {
      success_out!(out_error, bytes_written as i64);
    }
    Err(err) => {
      return_out!(out_error, err, u32);
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
  if rpi_image.is_null() {
    return_out!(out_error, Error::NullPointer, u32);
  }

  let Some(fat_path) = c_char_to_string(fat_path) else {
    return_out!(out_error, Error::InvalidArgument, u32);
  };
  let Some(string) = c_char_to_string(string) else {
    return_out!(out_error, Error::InvalidArgument, u32);
  };
  let rpi_image = unsafe { &mut *rpi_image };
  match rpi_image.write_bytes(&fat_path, string.as_bytes()) {
    Ok(bytes_written) => {
      success_out!(out_error, bytes_written as i64);
    }
    Err(err) => {
      return_out!(out_error, err, u32);
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
  if rpi_image.is_null() {
    return_out!(out_error, Error::NullPointer, u32);
  }
  if bytes_ptr.is_null() {
    return_out!(out_error, Error::NullPointer, u32);
  }

  let Some(fat_path) = c_char_to_string(fat_path) else {
    return_out!(out_error, Error::InvalidArgument, u32);
  };
  let bytes = unsafe { std::slice::from_raw_parts(bytes_ptr, bytes_len) };
  let rpi_image = unsafe { &mut *rpi_image };
  match rpi_image.write_bytes(&fat_path, bytes) {
    Ok(bytes_written) => {
      success_out!(out_error, bytes_written as i64);
    }
    Err(err) => {
      return_out!(out_error, err, u32);
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
  if rpi_image.is_null() {
    return_out!(out_error, Error::NullPointer, u32);
  }

  let Some(fat_path) = c_char_to_string(fat_path) else {
    return_out!(out_error, Error::InvalidArgument, u32);
  };
  let Some(string) = c_char_to_string(string) else {
    return_out!(out_error, Error::InvalidArgument, u32);
  };
  let rpi_image = unsafe { &mut *rpi_image };
  match rpi_image.append_bytes(&fat_path, string.as_bytes()) {
    Ok(bytes_written) => {
      success_out!(out_error, bytes_written as i64);
    }
    Err(err) => {
      return_out!(out_error, err, u32);
    }
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
  if rpi_image.is_null() {
    return_out!(out_error, Error::NullPointer, u32);
  }
  if bytes_ptr.is_null() {
    return_out!(out_error, Error::NullPointer, u32);
  }

  let Some(fat_path) = c_char_to_string(fat_path) else {
    return_out!(out_error, Error::InvalidArgument, u32);
  };
  let bytes = unsafe { std::slice::from_raw_parts(bytes_ptr, bytes_len) };
  let rpi_image = unsafe { &mut *rpi_image };
  match rpi_image.append_bytes(&fat_path, bytes) {
    Ok(bytes_written) => {
      success_out!(out_error, bytes_written as i64);
    }
    Err(err) => {
      return_out!(out_error, err, u32);
    }
  }
}

// pub fn save_to_file(self, file: impl AsRef<Path>) -> Result<(), Error>
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_save_to_file(rpi_image: *mut RpiImage, file: *const c_char) -> i64 {
  check_not_null!(rpi_image, -1);

  let Some(file) = c_char_to_pathbuf(file) else {
    #[cfg(feature = "ffi_debug")]
    set_last_error_message(Error::InvalidArgument.to_string());
    return Error::InvalidArgument.ffi() as i64;
  };
  let mut rpi_image = unsafe { Box::from_raw(rpi_image) };

  match rpi_image.save_to_file(file) {
    Err(err) => {
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(err.to_string());
      err.ffi() as i64
    }
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
  check_not_null!(rpi_image, -1);
  let Some(progress) = progress else {
    return -1;
  };

  let Some(file) = c_char_to_pathbuf(file) else {
    #[cfg(feature = "ffi_debug")]
    set_last_error_message(Error::InvalidArgument.to_string());
    return Error::InvalidArgument.ffi() as i64;
  };
  let mut rpi_image = unsafe { Box::from_raw(rpi_image) };

  let file = OpenOptions::new()
    .create(true)
    .truncate(true)
    .read(true)
    .write(true)
    .open(file);
  let file = match file {
    Ok(f) => f,
    Err(e) => {
      let e = Error::Io(e);
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(e.to_string());
      return e.ffi() as i64;
    }
  };

  let cb = |written| progress(written, context);
  let mut writer = ProgressWriter::new(file, cb);
  match rpi_image.save_to_writer(&mut writer) {
    Err(err) => {
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(err.to_string());
      err.ffi() as i64
    }
    Ok(_) => 0,
  }
}

// pub(crate) fn save_to_fd(self, fd: RawFd) -> Result<(), Error>
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_save_to_fd(rpi_image: *mut RpiImage, fd: i32) -> i64 {
  check_not_null!(rpi_image, -1);

  let mut rpi_image = unsafe { Box::from_raw(rpi_image) };
  let mut file = unsafe { File::from_raw_fd(fd) };

  match rpi_image.save_to_writer(&mut file) {
    Err(err) => {
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(err.to_string());
      err.ffi() as i64
    }
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
  check_not_null!(rpi_image, -1);
  let Some(progress) = progress else {
    return -1;
  };

  let mut rpi_image = unsafe { Box::from_raw(rpi_image) };
  let file = unsafe { File::from_raw_fd(fd) };
  let cb = |written| progress(written, context);
  let mut writer = ProgressWriter::new(file, cb);

  match rpi_image.save_to_writer(&mut writer) {
    Err(err) => {
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(err.to_string());
      err.ffi() as i64
    }
    Ok(_) => 0,
  }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_verify_file(rpi_image: *mut RpiImage, file: *const c_char) -> i64 {
  check_not_null!(rpi_image, -1);

  let Some(file) = c_char_to_pathbuf(file) else {
    #[cfg(feature = "ffi_debug")]
    set_last_error_message(Error::InvalidArgument.to_string());
    return Error::InvalidArgument.ffi() as i64;
  };
  let mut rpi_image = unsafe { Box::from_raw(rpi_image) };

  match rpi_image.verify_file(file) {
    Err(err) => {
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(err.to_string());
      err.ffi() as i64
    }
    Ok(false) => {
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(Error::CopyMismatch.to_string());
      Error::CopyMismatch.ffi() as i64
    }
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
  check_not_null!(rpi_image, -1);
  let Some(progress) = progress else {
    return -1;
  };

  let Some(file) = c_char_to_pathbuf(file) else {
    #[cfg(feature = "ffi_debug")]
    set_last_error_message(Error::InvalidArgument.to_string());
    return Error::InvalidArgument.ffi() as i64;
  };
  let mut rpi_image = unsafe { Box::from_raw(rpi_image) };
  let file = match File::open(file) {
    Ok(f) => f,
    Err(err) => {
      let err = Error::Io(err);
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(err.to_string());
      return err.ffi() as i64;
    }
  };
  let cb = |written| progress(written, context);
  let mut reader = ProgressReader::new(file, cb);

  match rpi_image.verify_reader(&mut reader) {
    Err(err) => {
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(err.to_string());
      err.ffi() as i64
    }
    Ok(false) => {
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(Error::CopyMismatch.to_string());
      Error::CopyMismatch.ffi() as i64
    }
    Ok(true) => 0,
  }
}

// pub(crate) fn save_to_fd(self, fd: RawFd) -> Result<(), Error>
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_verify_fd(rpi_image: *mut RpiImage, fd: i32) -> i64 {
  check_not_null!(rpi_image, -1);

  let mut rpi_image = unsafe { Box::from_raw(rpi_image) };
  let mut file = unsafe { File::from_raw_fd(fd) };

  match rpi_image.verify_reader(&mut file) {
    Err(err) => {
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(err.to_string());
      err.ffi() as i64
    }
    Ok(false) => {
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(Error::CopyMismatch.to_string());
      Error::CopyMismatch.ffi() as i64
    }
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
  check_not_null!(rpi_image, -1);
  let Some(progress) = progress else {
    return -1;
  };

  let mut rpi_image = unsafe { Box::from_raw(rpi_image) };
  let file = unsafe { File::from_raw_fd(fd) };
  let cb = |read| progress(read, context);
  let mut reader = ProgressReader::new(file, cb);

  match rpi_image.verify_reader(&mut reader) {
    Err(err) => {
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(err.to_string());
      err.ffi() as i64
    }
    Ok(false) => {
      #[cfg(feature = "ffi_debug")]
      set_last_error_message(Error::CopyMismatch.to_string());
      Error::CopyMismatch.ffi() as i64
    }
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
