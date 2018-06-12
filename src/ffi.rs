
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

macro_rules! c_str {
    ($c_str: ident) => {
        unsafe { CStr::from_ptr($c_str).to_string_lossy() };
    };
}

#[no_mangle]
pub extern "C" fn reset_default_theme_name(theme: *const c_char) {

    let theme = c_str!(theme);

    icon_lookup::reset_default_theme(theme);
}

#[no_mangle]
pub extern "C" fn find_icon_with_theme_name(theme: *const c_char, icon: *const c_char, size: i32, scale: i32) -> *const c_char {

    let theme = c_str!(theme);
    let icon = c_str!(icon);

    c_strify!(icon_lookup::find_icon_with_theme_name(theme, icon, size, scale))
}

#[no_mangle]
pub extern "C" fn find_icon(icon: *const c_char, size: i32, scale: i32) -> *const c_char {

    let icon = c_str!(icon);

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