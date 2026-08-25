use std::{cell::RefCell, ffi::*, mem::ManuallyDrop, ptr::{slice_from_raw_parts_mut}, rc::Rc};

use otr_core::{Result, SystemError, value::Value};

#[repr(C)]
pub struct CValue {
    data: CValueUnion,
    tag: CType,
}

#[repr(C)]
pub union CValueUnion {
    integer: i64,
    float: f64,
    string: *mut c_char,
    char: c_char,
    bool: bool,
    array: ManuallyDrop<CArray>,
}

#[repr(C)]
pub struct CArray {
    data: *mut CValue,
    len: usize,
}

#[repr(C)]
pub enum CType {
    Null,
    Integer,
    Float,
    String,
    Char,
    Bool,
    Array,
}

impl TryFrom<Value> for CValue {
    type Error = Box<dyn otr_core::Error>;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Null => Ok(CValue { data: CValueUnion { integer: 0 }, tag: CType::Null }),
            Value::Integer(integer) => Ok(CValue { data: CValueUnion { integer }, tag: CType::Integer }),
            Value::Float(float) => Ok(CValue { data: CValueUnion { float }, tag: CType::Float }),
            Value::String(string) => string_to_cstring(string),
            Value::Char(c) => char_to_cchar(c),
            Value::Bool(bool) => Ok(CValue { data: CValueUnion { bool }, tag: CType::Bool }),
            Value::Array(array) => array_to_carray(array),
            Value::ArrayRef(_) => Err(
                SystemError::new("Only owned arrays can be exposed to external functions!".into()).boxed()
            ),
            Value::Struct(_) | Value::StructRef(_) => Err(
                SystemError::new("Structs cannot be exposed to external functions!".into()).boxed()
            ),
            Value::Type(_) => Err(
                SystemError::new("Otr's types cannot be exposed to external functions!".into()).boxed()
            ),
        }
    }
}

pub unsafe fn cvalue_to_value(cvalue: CValue) -> Result<Value> {
    match cvalue.tag {
        CType::Null => Ok(Value::Null),
        CType::Integer => Ok(Value::Integer(unsafe { cvalue.data.integer })),
        CType::Float => Ok(Value::Float(unsafe { cvalue.data.float })),
        CType::String => {
            let cstring = unsafe { CString::from_raw(cvalue.data.string) };

            let string = cstring.into_string()
                .map_err(|_| SystemError::new("CString did not contain valid UTF-8 and thus could not be converted to an OTR value!".into()).boxed())?;

            Ok(Value::String(string))
        },
        CType::Char => Ok(Value::Char(
            char::from_u32(unsafe { cvalue.data.char } as u32)
                .ok_or_else(|| SystemError::new("CChar did not contain a valid character!".into()).boxed())?
        )),
        CType::Bool => Ok(Value::Bool(unsafe { cvalue.data.bool })),
        CType::Array => {
            let slice = unsafe { Box::from_raw(slice_from_raw_parts_mut(cvalue.data.array.data, cvalue.data.array.len)) };
            
            let mut otr_slice = Vec::with_capacity(slice.len()).into_boxed_slice();
            for (i, item) in slice.into_iter().enumerate() {
                otr_slice[i] = unsafe { cvalue_to_value(item) }?;
            }

            Ok(Value::Array(Rc::new(RefCell::new(Some(otr_slice)))))
        },
    }
}

pub fn string_to_cstring(string: String) -> Result<CValue> {
    if let Ok(cstring) = CString::new(string) {
        Ok(CValue { data: CValueUnion { string: cstring.into_raw() }, tag: CType::String })
    } else {
        Err(
            SystemError::new("String contains 'Nul' characters and thus cannot be exposed to external functions!".into()).boxed()
        )
    }
}
#[unsafe(export_name = "OTR_FREE_CSTRING")]
pub unsafe extern "C" fn free_cstring(string: *mut c_char) {
    if string.is_null() {
        panic!("OTR_FREE_CSTRING received nullptr!");
    }

    let cstring = unsafe { CString::from_raw(string) };

    drop(cstring);
}

pub fn char_to_cchar(char: char) -> Result<CValue> {
    if let Ok(cchar) = u8::try_from(char) {
        Ok(CValue { data: CValueUnion { char: cchar as c_char }, tag: CType::Char })
    } else {
        Err(
            SystemError::new("Character cannot be represented as a single byte and thus cannot be exposed to external functions!".into()).boxed()
        )
    }
}

pub fn array_to_carray(array: Rc<RefCell<Option<Box<[Value]>>>>) -> Result<CValue> {
    let array = Rc::try_unwrap(array)
        .map_err(|_|
            SystemError::new("Array has more than one owner and thus cannot be exposed to external functions!".into()).boxed()
        )?
        .into_inner();
    

    if let Some(array) = array {
        let mut carray = Vec::with_capacity(array.len()).into_boxed_slice();
        for (i, item) in array.into_iter().enumerate() {
            carray[i] = CValue::try_from(item)?;
        }

        let carray = ManuallyDrop::new(CArray {
            len: carray.len(),
            data: Box::into_raw(carray) as *mut CValue,
        });

        Ok(CValue { data: CValueUnion { array: carray }, tag: CType::Array })
    } else {
        Err(
            SystemError::new("Array moved and thus cannot be expsoed to external functions!".into()).boxed()
        )
    }
}

#[unsafe(export_name = "OTR_FREE_CARRAY")]
pub unsafe extern "C" fn free_carray(array: *mut CValue, len: usize) {
    let slice = unsafe { Box::from_raw(slice_from_raw_parts_mut(array, len)) };

    drop(slice);
}

#[unsafe(export_name = "OTR_NULL")]
pub unsafe extern "C" fn null() -> CValue {
    CValue { data: CValueUnion { integer: 0 }, tag: CType::Null }
}

#[unsafe(export_name = "OTR_INTEGER")]
pub unsafe extern "C" fn integer(integer: i64) -> CValue {
    CValue { data: CValueUnion { integer }, tag: CType::Integer }
}

#[unsafe(export_name = "OTR_FLOAT")]
pub unsafe extern "C" fn float(float: f64) -> CValue {
    CValue { data: CValueUnion { float }, tag: CType::Float }
}

#[unsafe(export_name = "OTR_STRING")]
pub unsafe extern "C" fn string(string: *mut c_char) -> CValue {
    CValue { data: CValueUnion { string }, tag: CType::String }
}

#[unsafe(export_name = "OTR_CHAR")]
pub unsafe extern "C" fn char(char: i8) -> CValue {
    CValue { data: CValueUnion { char }, tag: CType::Char }
}

#[unsafe(export_name = "OTR_BOOL")]
pub unsafe extern "C" fn bool(bool: bool) -> CValue {
    CValue { data: CValueUnion { bool }, tag: CType::Bool }
}

#[unsafe(export_name = "OTR_ARRAY")]
pub unsafe extern "C" fn array(array: *mut CValue, len: usize) -> CValue {
    CValue { data: CValueUnion { array: ManuallyDrop::new(CArray { data: array, len }) }, tag: CType::Integer }
}