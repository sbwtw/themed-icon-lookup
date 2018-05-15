
use icon_lookup;

use std::mem;
use std::ptr;
use std::os::raw::c_char;
use std::ffi::{CStr, CString};

macro_rules! c_strify {
    ($pathbuf: expr) => {
        match $pathbuf {
            Some(path) => {
                let icon = CString::new(path.to_string_lossy().as_ref()).unwrap();
                let ptr = icon.as_ptr();

                mem::forget(icon);

                ptr
            },
            None => ptr::null()
        }
    };
}

#[no_mangle]
pub extern "C" fn find_icon_with_theme_name(theme: *const c_char, icon: *const c_char, size: i32, scale: i32) -> *const c_char {

    let theme = unsafe { CStr::from_ptr(theme).to_string_lossy() };
    let icon = unsafe { CStr::from_ptr(icon).to_string_lossy() };

    c_strify!(icon_lookup::find_icon_with_theme_name(theme, icon, size, scale))
}

#[no_mangle]
pub extern "C" fn find_icon(icon: *const c_char, size: i32, scale: i32) -> *const c_char {

    let icon = unsafe { CStr::from_ptr(icon).to_string_lossy() };

    c_strify!(icon_lookup::find_icon(icon, size, scale))
}

#[no_mangle]
pub extern "C" fn free_cstring(cstring: *mut c_char) {
    if !cstring.is_null() {
        unsafe { CString::from_raw(cstring); }
    }
}

#[cfg(test)]
mod test {

    use ffi::*;

    use std::ptr;

    #[test]
    fn test_null_ptr() {
        let nullptr: *mut i8 = ptr::null_mut();

        // should't be crashed
        free_cstring(nullptr);
    }
}