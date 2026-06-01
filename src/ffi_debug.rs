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
pub extern "C" fn rpi_imgpatcher_last_error_message() -> *mut c_char {
  LAST_ERROR_MESSAGE.with(|slot| {
    slot
      .borrow()
      .as_ref()
      .map(|msg| {
        let bytes = msg.as_bytes_with_nul();
        unsafe {
          let ptr = libc::malloc(bytes.len()) as *mut c_char;
          if !ptr.is_null() {
            std::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, ptr, bytes.len());
          }
          ptr
        }
      })
      .unwrap_or(std::ptr::null_mut())
  })
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn rpi_imgpatcher_last_error_free(error: *mut c_char) {
  if error.is_null() {
    return;
  }

  unsafe {
    libc::free(error as *mut libc::c_void);
  }
}
