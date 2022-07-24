use std::ffi::CStr;
use std::os::raw::c_char;

pub fn c_str_from_char_slice(raw: &[c_char]) -> &CStr {
    unsafe {
        CStr::from_ptr( raw.as_ptr() )
    }
}
