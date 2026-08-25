use std::{cell::RefCell, rc::Rc};

use otr_core::{value::Value, vec_map::VecMap, Result};

use crate::{error::RuntimeError, module::Module, procedures::{Procedure, RuntimeProcedure}};

#[derive(Debug, Clone, Copy)]
pub struct RuntimeExternalFunction {
    function: otr_ffi::ExternalFunctionPointer,
    num_args: usize,
}

impl Procedure for RuntimeExternalFunction {
    fn call(&self, _environment: crate::environment::Environment, arguments: Vec<otr_core::value::Value>) -> otr_core::Result<otr_core::value::Value> {
        let packed_arguments = Value::Array(Rc::new(RefCell::new(Some(arguments.into_boxed_slice()))));

        unsafe {
            let returned = (self.function)(packed_arguments.try_into()?);
            
            otr_ffi::cvalue_to_value(returned)
        }
    }

    fn get_num_args(&self) -> usize {
        self.num_args
    }

    fn get_stack_size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub struct ExternalModule {
    definition: otr_ffi::external::ExternalModule,
    bindings: VecMap<String, RuntimeExternalFunction>,
}

impl ExternalModule {
    pub fn new(definition: otr_ffi::external::ExternalModule) -> Self {
        Self { definition, bindings: VecMap::default() }
    }

    pub fn insert_binding(&mut self, symbol_name: String, function: otr_ffi::ExternalFunctionPointer) -> Result<Option<RuntimeExternalFunction>> {
        let definition = self.definition
            .functions
            .get(&symbol_name)
            .ok_or_else(|| RuntimeError::Unknown {
                message: format!("Could not insert binding '{symbol_name}' as it was not found in the difinition!")
            }.boxed())?;
        
        Ok(self.bindings.insert(symbol_name, RuntimeExternalFunction { function, num_args: definition.parameters.len() }))
    }

    pub fn get_binding(&self, symbol_name: impl AsRef<str>) -> Option<RuntimeExternalFunction> {
        self.bindings.get(symbol_name).copied()
    }
}

impl Module for ExternalModule {
    fn get_procedure(
        &'_ self,
        identifier: &str,
        _private_access: bool,
    ) -> otr_core::Result<crate::procedures::RuntimeProcedure<'_>> {
        self.bindings
            .get(identifier)
            .ok_or_else(|| RuntimeError::NoSuchMember { member_identifier: identifier.to_string() }.boxed())
            .map(|binding| RuntimeProcedure::AbstractRef(binding))
    }

    fn get_associated_procedure(
        &'_ self,
        struct_identifier: &str,
        procedure_identifier: &str,
        _private_access: bool,
    ) -> otr_core::Result<crate::procedures::RuntimeProcedure<'_>> {
        Err(RuntimeError::NoSuchMember { member_identifier: format!("{struct_identifier}->{procedure_identifier}") }.boxed())
    }

    fn get_struct(&self, identifier: &str, _private_access: bool) -> otr_core::Result<otr_core::r#struct::Struct> {
        Err(RuntimeError::NoSuchMember { member_identifier: identifier.to_string() }.boxed())
    }

    fn get_operation(&self, struct_identifier: &str, operator: otr_core::expression::Operator, _private_access: bool) -> otr_core::Result<crate::procedures::RuntimeProcedure<'_>> {
        Err(RuntimeError::OperatorNotOverloaded { struct_identifier: struct_identifier.to_string(), operator }.boxed())
    }
}