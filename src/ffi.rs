
use icon_lookup;

use std::mem;
use std::os::raw::c_char;
use std::ffi::{CStr, CString};

#[no_mangle]
pub extern "C" fn find_icon_with_theme_name(theme: *const c_char, icon: *const c_char, size: i32, scale: i32) -> *const c_char {

    let theme = unsafe { CStr::from_ptr(theme).to_string_lossy() };
    let icon = unsafe { CStr::from_ptr(icon).to_string_lossy() };

    let path = match icon_lookup::find_icon_with_theme_name(theme, icon, size, scale) {
        Some(path) => {
            CString::new(path.to_string_lossy().as_ref()).unwrap()
        },
        _ => CString::new("").unwrap(),
    };

    let ptr = path.as_ptr();
    mem::forget(path);

    ptr
}

#[no_mangle]
pub extern "C" fn free_cstring(cstring: *mut c_char) {
    unsafe { CString::from_raw(cstring); }
}