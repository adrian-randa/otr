use std::path::PathBuf;

use colored::Colorize;

use crate::runtime::{ModuleAddress, module::Module};

use super::Error;

pub(crate) struct ProcedureContextDecorator {
    error: Box<dyn Error>,
    procedure_id: ModuleAddress
}

impl Error for ProcedureContextDecorator {}

impl std::fmt::Display for ProcedureContextDecorator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let context = format!("In procedure {}::{}", self.procedure_id.get_module_id(), self.procedure_id.get_identifier());

        write!(f, "{}\n\t{}", self.error, (&context as &str).bright_red())
    }
}

impl ProcedureContextDecorator {
    pub(crate) fn new_boxed(error: Box<dyn Error>, procedure_id: ModuleAddress) -> Box<dyn Error> {
        Box::new(Self {
            error, procedure_id
        })
    }
}


pub(crate) struct AssociatedProcedureContextDecorator {
    error: Box<dyn Error>,
    struct_id: ModuleAddress,
    procedure_identifier: String, 
}

impl Error for AssociatedProcedureContextDecorator {}

impl std::fmt::Display for AssociatedProcedureContextDecorator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let context = format!("In associated procedure {}::{}->{}",
            self.struct_id.get_module_id(),
            self.struct_id.get_identifier(),
            self.procedure_identifier
        );

        write!(f, "{}\n\t{}", self.error, (&context as &str).bright_red())
    }
}

impl AssociatedProcedureContextDecorator {
    pub(crate) fn new_boxed(error: Box<dyn Error>, struct_id: ModuleAddress, procedure_identifier: String) -> Box<dyn Error> {
        Box::new(Self {
            error, struct_id, procedure_identifier
        })
    }
}


pub(crate) struct SourceFileContextDecorator {
    pub(crate) error: Box<dyn Error>,

    pub(crate) path: PathBuf,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl Error for SourceFileContextDecorator {}

impl std::fmt::Display for SourceFileContextDecorator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = format!("Occurred in file {:?} on line {}:{}.", self.path, self.line, self.column);

        write!(f, "{}\n{}", self.error, (&message as &str).bright_black())
    }
}

impl SourceFileContextDecorator {
    pub(crate) fn boxed(self) -> Box<dyn Error> {
        Box::new(self)
    }
}

pub(crate) struct HintContextDecorator {
    pub(crate) error: Box<dyn Error>,

    pub(crate) message: String,
}

impl Error for HintContextDecorator {}

impl std::fmt::Display for HintContextDecorator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{} {}", self.error, "Hint:".on_blue(), (&self.message as &str).blue())
    }
}

impl HintContextDecorator {
    pub(crate) fn boxed(self) -> Box<dyn Error> {
        Box::new(self)
    }
}