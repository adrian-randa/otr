use std::{ffi::{CString, c_char}, ptr::slice_from_raw_parts_mut};

use crate::value::{CValue, cvalue_to_value};

#[unsafe(export_name = "OTR_FREE_CSTRING")]
pub unsafe extern "C" fn free_cstring(string: *mut c_char) {
    if string.is_null() {
        panic!("OTR_FREE_CSTRING received nullptr!");
    }

    let cstring = unsafe { CString::from_raw(string) };

    drop(cstring);
}

#[unsafe(export_name = "OTR_FREE_CARRAY")]
pub unsafe extern "C" fn free_carray(array: *mut CValue, len: usize) {
    let slice = unsafe { Box::from_raw(slice_from_raw_parts_mut(array, len)) };

    drop(slice);
}

#[unsafe(export_name = "OTR_FREE_CVALUE")]
pub unsafe extern "C" fn free_cvalue(cvalue: *mut CValue) {
    let cvalue = unsafe { Box::from_raw(cvalue) };

    drop(cvalue);
}

#[unsafe(export_name = "OTR_FREE_CVALUE_RECURSIVE")]
pub unsafe extern "C" fn free_cvalue_recursive(cvalue: *mut CValue) {
    let cvalue = unsafe { Box::from_raw(cvalue) };

    drop(unsafe { cvalue_to_value(*cvalue) }.unwrap())
}