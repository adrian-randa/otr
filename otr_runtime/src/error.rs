use std::{cell::RefCell, rc::Rc};

use colored::Colorize;

use otr_core::{error::Error, expression::Operator, module::ModuleAddress, r#struct::Struct, r#type::Type, value::Value};

#[derive(Debug)]
pub(crate) struct ValueError {
    value: Value,
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Error for ValueError {
    fn to_value(&self) -> Value {
        self.value.clone()
    }
}

impl ValueError {
    pub(crate) fn new(value: Value) -> Self {
        Self { value }
    }

    pub(crate) fn boxed(self) -> Box<dyn Error> {
        Box::new(self)
    }
}

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
    #[allow(unused)]
    KeyAlreadyPresent {
        key: String,
    },
    NoEntrypoint,
    TypeMismatch {
        expected: Type,
        found: Type,
    },
    NoSuchVariable {
        variable_identifier: String,
    },
    InvalidNumberOfArgs {
        expected: usize,
        supplied: usize,
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
    OperatorNotOverloaded {
        struct_identifier: String,
        operator: Operator,
    },
    OperatorNotExported {
        struct_identifier: String,
        operator: Operator,
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
                format!("Index out of bounds! Tried to index element at {index}, but the array size was {array_length}."),
            RuntimeError::NoSuchMember { member_identifier } => 
                format!("Tried to get member '{member_identifier}', but no such member exists!"),
            RuntimeError::UseOfMovedValue => 
                "Use of moved value!".to_string(),
            RuntimeError::UseOfDroppedValue => 
                "Use of dropped value!".to_string(),
            RuntimeError::CannotReference { ty } => 
                format!("Referencing values of type {ty} is not allowed!"),
            RuntimeError::FieldIsPrivate => 
                "Tried to access private field!".to_string(),
            RuntimeError::KeyAlreadyPresent { key } => 
                format!("The key '{key}' is already present!"),
            RuntimeError::NoEntrypoint => 
                "Entrypoint is not specified!".to_string(),
            RuntimeError::TypeMismatch { expected, found } => 
                format!("Type mismatch! Expected {expected} but found {found}."),
            RuntimeError::NoSuchVariable { variable_identifier } => 
                format!("Variable '{variable_identifier}' is not defined!"),
            RuntimeError::InvalidNumberOfArgs { expected, supplied } => 
                format!("Invalid number of arguments! This procedure takes {expected} arguments, but {supplied} were supplied."),
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
            RuntimeError::OperatorNotOverloaded { struct_identifier, operator } =>
                format!("Operator '{operator}' not overloaded for {struct_identifier}!"),
            RuntimeError::OperatorNotExported { struct_identifier, operator } =>
                format!("Operator '{operator}' for {struct_identifier} not exported!"),
            RuntimeError::ModuleNotLoaded { module_identifier } => 
                format!("Module '{module_identifier}' is not loaded!"),
            RuntimeError::Unknown { message } => 
                message.to_string(),
            RuntimeError::DivisionByZero => 
                "Division by zero!".to_string(),
        }
    }
}

impl Error for RuntimeError {
    fn to_value(&self) -> Value {
        let err = |variant: &str| {
            Value::Struct(Rc::new(RefCell::new(Some(
                Struct::new(ModuleAddress::new("Errors".into(), variant.to_string() + "Error"))
                    .with_member("message".into(), Value::String(self.get_message()), false)
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
            RuntimeError::NoSuchVariable { variable_identifier: _ } => err("NoSuchVariable"),
            RuntimeError::ProcedureNotExported { procedure_identifier: _ } => err("NotExported"),
            RuntimeError::AssociatedProcedureNotExported { procedure_identifier: _, struct_identifier: _ } => err("NotExported"),
            RuntimeError::StructNotExported { struct_identifier: _ } => err("NotExported"),
            RuntimeError::ProcedureNotDefined { procedure_identifier: _ } => err("NotDefined"),
            RuntimeError::StructNotDefined { struct_identifier: _ } => err("NotDefined"),
            RuntimeError::AssociatedProcedureNotDefined { procedure_identifier: _, struct_identifier: _ } => err("NotDefined"),
            RuntimeError::OperatorNotOverloaded { struct_identifier: _, operator: _ } => err("OperatorNotOverloaded"),
            RuntimeError::OperatorNotExported { struct_identifier: _, operator: _ } => err("OperatorNotExported"),
            RuntimeError::ModuleNotLoaded { module_identifier: _ } => err("ModuleNotLoaded"),
            RuntimeError::Unknown { message: _ } => err("Unknown"),
            RuntimeError::DivisionByZero => err("DivisionByZero"),
            RuntimeError::InvalidNumberOfArgs { expected: _, supplied: _ } => err("InvalidNumberOfArgs"),
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
    pub(crate) fn boxed(self) -> Box<dyn Error> {
        Box::new(self)
    }
}

pub(crate) mod context;