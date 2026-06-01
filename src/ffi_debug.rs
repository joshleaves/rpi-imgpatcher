use std::cell::RefCell;
use std::ffi::CString;
use std::ffi::c_char;

thread_local! {
  static LAST_ERROR_MESSAGE: RefCell<Option<CString>> = const { RefCell::new(None) };
}

pub fn set_last_error_message(message: impl Into<String>) {
  let message = message.into().replace('\0', " ");
  let cstring = CString::new(message).unwrap_or_else(|_| CString::new("unknown error").unwrap());

  LAST_ERROR_MESSAGE.with(|slot| {
    *slot.borrow_mut() = Some(cstring);
  });
}

/// Returns the last error message as a heap-allocated null-terminated string.
///
/// The caller is responsible for freeing the returned string using
/// `rpi_imgpatcher_last_error_free`.
/// Returns NULL if no error message is available.
#[unsafe(no_mangle)]
pub extern "C" fn rpi_imgpatcher_last_error_message() -> *mut c_char {
  LAST_ERROR_MESSAGE.with(|slot| {
    slot
      .borrow()
      .as_ref()
      .map(|msg| msg.to_owned().into_raw())
      .unwrap_or(std::ptr::null_mut())
  })
}

/// Frees an error message returned by `rpi_imgpatcher_last_error_message`.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_imgpatcher_last_error_free(error: *mut c_char) {
  if error.is_null() {
    return;
  }

  unsafe {
    std::mem::drop(CString::from_raw(error));
  }
}
