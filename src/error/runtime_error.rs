use std::{cell::RefCell, rc::Rc};

use colored::Colorize;

use crate::runtime::{ModuleAddress, Struct, Type, Value};

#[derive(Debug)]
pub enum RuntimeError {
    IndexingNotAccepted {
        ty: Type,
    },
    MembersNotAccepted {
        ty: Type,
    },
    AddressantsNotAccepted {
        ty: Type,
    },
    IndexOutOfBounds {
        array_length: usize,
        index: usize,
    },
    NoSuchMember {
        member_identifier: String,
    },
    UseOfMovedValue,
    UseOfDroppedValue,
    CannotReference {
        ty: Type,
    },
    FieldIsPrivate,
    KeyAlreadyPresent {
        key: String,
    },
    NoEntrypoint,
    TypeMismatch {
        expected: Type,
        found: Type,
    },
    VariableAlreadyPresent {
        variable_identifier: String,
    },
    NoSuchVariable {
        variable_identifier: String,
    },
    ProcedureNotExported {
        procedure_identifier: String,
    },
    AssociatedProcedureNotExported {
        procedure_identifier: String,
        struct_identifier: String,
    },
    StructNotExported {
        struct_identifier: String,
    },
    ProcedureNotDefined {
        procedure_identifier: String,
    },
    StructNotDefined {
        struct_identifier: String,
    },
    AssociatedProcedureNotDefined {
        procedure_identifier: String,
        struct_identifier: String,
    },
    ModuleNotLoaded {
        module_identifier: String,
    },
    DivisionByZero,

    Unknown {
        message: String,
    },
}

impl RuntimeError {
    pub(crate) fn get_message(&self) -> String {
        match self {
            RuntimeError::IndexingNotAccepted { ty } => 
                format!("Indexing not allowed on values of type {ty}!"),
            RuntimeError::MembersNotAccepted { ty } => 
                format!("Getting a member is not allowed on values of type {ty}!"),
            RuntimeError::AddressantsNotAccepted { ty } => 
                format!("Values of type {ty} do not accept addressants!"),
            RuntimeError::IndexOutOfBounds { array_length, index } => 
                format!("Index out of bounds! Tried to index element at {array_length}, but the array size was {index}."),
            RuntimeError::NoSuchMember { member_identifier } => 
                format!("Tried to get member '{member_identifier}', but no such member exists!"),
            RuntimeError::UseOfMovedValue => 
                format!("Use of moved value!"),
            RuntimeError::UseOfDroppedValue => 
                format!("Use of dropped value!"),
            RuntimeError::CannotReference { ty } => 
                format!("Referencing values of type {ty} is not allowed!"),
            RuntimeError::FieldIsPrivate => 
                format!("Tried to access private field!"),
            RuntimeError::KeyAlreadyPresent { key } => 
                format!("The key '{key}' is already present!"),
            RuntimeError::NoEntrypoint => 
                format!("Entrypoint is not specified!"),
            RuntimeError::TypeMismatch { expected, found } => 
                format!("Type mismatch! Expected {expected} but found {found}."),
            RuntimeError::VariableAlreadyPresent { variable_identifier } => 
                format!("Variable '{variable_identifier}' is already declared!"),
            RuntimeError::NoSuchVariable { variable_identifier } => 
                format!("Variable '{variable_identifier}' is not defined!"),
            RuntimeError::ProcedureNotExported { procedure_identifier } => 
                format!("Procedure '{procedure_identifier}' is not exported!"),
            RuntimeError::AssociatedProcedureNotExported { procedure_identifier, struct_identifier } => 
                format!("Associated procedure '{struct_identifier}->{procedure_identifier}' is not exported!"),
            RuntimeError::StructNotExported { struct_identifier } => 
                format!("Struct '{struct_identifier}' is not exported!"),
            RuntimeError::ProcedureNotDefined { procedure_identifier } => 
                format!("Procedure '{procedure_identifier}' is not defined!"),
            RuntimeError::StructNotDefined { struct_identifier } => 
                format!("Struct '{struct_identifier}' is not defined!"),
            RuntimeError::AssociatedProcedureNotDefined { procedure_identifier, struct_identifier } => 
                format!("Associated procedure '{struct_identifier}->{procedure_identifier}' is not defined!"),
            RuntimeError::ModuleNotLoaded { module_identifier } => 
                format!("Module '{module_identifier}' is not loaded!"),
            RuntimeError::Unknown { message } => 
                format!("{message}"),
            RuntimeError::DivisionByZero => 
                format!("Division by zero!"),
        }
    }
}

impl super::Error for RuntimeError {
    fn to_value(&self) -> Value {
        let err = |variant: &str| {
            Value::Struct(Rc::new(RefCell::new(Some(
                Struct::new(ModuleAddress::new("Errors".into(), variant.to_string() + "Error"))
                    .with_member("message".into(), Value::String(self.get_message()), false).unwrap()
            ))))
        };

        match self {
            RuntimeError::IndexingNotAccepted { ty: _ } => err("IndexingNotAccepted"),
            RuntimeError::MembersNotAccepted { ty: _ } => err("MembersNotAccepted"),
            RuntimeError::AddressantsNotAccepted { ty: _ } => err("AddressantsNotAccepted"),
            RuntimeError::IndexOutOfBounds { array_length: _, index: _ } => err("IndexOutOfBounds"),
            RuntimeError::NoSuchMember { member_identifier: _ } => err("NoSuchMember"),
            RuntimeError::UseOfMovedValue => err("UseOfMovedValue"),
            RuntimeError::UseOfDroppedValue => err("UseOfDroppedValue"),
            RuntimeError::CannotReference { ty: _ } => err("CannotReference"),
            RuntimeError::FieldIsPrivate => err("FieldIsPrivate"),
            RuntimeError::KeyAlreadyPresent { key: _ } => err("KeyAlreadyPresent"),
            RuntimeError::NoEntrypoint => err("NoEntrypoint"),
            RuntimeError::TypeMismatch { expected: _, found: _ } => err("TypeMismatch"),
            RuntimeError::VariableAlreadyPresent { variable_identifier: _ } => err("VariableAlreadyPresent"),
            RuntimeError::NoSuchVariable { variable_identifier: _ } => err("NoSuchVariable"),
            RuntimeError::ProcedureNotExported { procedure_identifier: _ } => err("NotExported"),
            RuntimeError::AssociatedProcedureNotExported { procedure_identifier: _, struct_identifier: _ } => err("NotExported"),
            RuntimeError::StructNotExported { struct_identifier: _ } => err("NotExported"),
            RuntimeError::ProcedureNotDefined { procedure_identifier: _ } => err("NotDefined"),
            RuntimeError::StructNotDefined { struct_identifier: _ } => err("NotDefined"),
            RuntimeError::AssociatedProcedureNotDefined { procedure_identifier: _, struct_identifier: _ } => err("NotDefined"),
            RuntimeError::ModuleNotLoaded { module_identifier: _ } => err("ModuleNotLoaded"),
            RuntimeError::Unknown { message: _ } => err("Unknown"),
            RuntimeError::DivisionByZero => err("DivisionByZero"),
        }
    }
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = self.get_message();

        write!(f, "{} {}", "Runtime Error:".on_red(), (&message as &str).red())
    }
}

impl RuntimeError {
    pub(crate) fn boxed(self) -> Box<dyn super::Error> {
        Box::new(self)
    }
}
