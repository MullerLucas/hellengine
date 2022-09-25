use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub fn c_str_from_char_slice(raw: &[c_char]) -> &CStr {
    unsafe {
        CStr::from_ptr( raw.as_ptr() )
    }
}

pub fn c_char_from_str_slice(slice: &[&str]) -> (Vec<CString>, Vec<*const c_char>) {
    let owned_data: Vec<_> = slice.iter()
        .map(|n| std::ffi::CString::new(*n).unwrap())
        .collect();
    let referenced_data: Vec<_> = owned_data.iter()
        .map(|n| n.as_ptr())
        .collect();

    (owned_data, referenced_data)
}
