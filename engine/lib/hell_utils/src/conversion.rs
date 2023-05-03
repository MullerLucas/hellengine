use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use hell_core::error::{HellResult, ErrToHellErr};




pub fn c_str_from_char_slice(raw: &[c_char]) -> &CStr {
    unsafe {
        CStr::from_ptr( raw.as_ptr() )
    }
}

pub fn c_char_from_str_slice(slice: &[&str]) -> HellResult<(Vec<CString>, Vec<*const c_char>)> {
    let owned_data: HellResult<Vec<_>> = slice.iter()
        .map(|n| std::ffi::CString::new(*n).to_generic_hell_err())
        .collect();
    let owned_data = owned_data?;
    let referenced_data: Vec<_> = owned_data.iter()
        .map(|n| n.as_ptr())
        .collect();

    Ok((owned_data, referenced_data))
}
