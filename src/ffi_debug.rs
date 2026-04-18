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

#[unsafe(no_mangle)]
pub extern "C" fn rpi_imgpatcher_last_error_message() -> *const c_char {
  LAST_ERROR_MESSAGE.with(|slot| {
    slot
      .borrow()
      .as_ref()
      .map(|msg| msg.as_ptr())
      .unwrap_or(std::ptr::null())
  })
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_image_last_error_free(error: *mut c_char) {
  if error.is_null() {
    return;
  }

  unsafe {
    std::mem::drop(Box::from_raw(error));
  }
}
