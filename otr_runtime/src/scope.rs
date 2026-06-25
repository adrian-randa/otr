use derive_more::{Deref, IntoIterator};


use otr_core::{expression::variable::{VariableAddress, VariableAddressant}, value::Value};
use crate::{error::RuntimeError, environment::Environment, expressions::eval_expression, value};

use otr_core::error::Result;



pub(crate) fn try_bake_variable_address(address: VariableAddress, environment: &Environment) -> Result<BakedVariableAddress> {

    let mut address = address.0;

    for i in 0..address.len() {
        if let VariableAddressant::DynamicIndex(expression) = &address[i] {
            let value = eval_expression(expression, environment)?;
            let idx: usize = match value {
                Value::Integer(value) => {
                    

                    value.try_into().unwrap()
                }
                _ => {
                    return Err(RuntimeError::TypeMismatch {
                        expected: otr_core::r#type::Type::Integer,
                        found: value.get_type_id(),
                    }
                    .boxed())
                }
            };

            address[i] = VariableAddressant::Index(idx);
        }
    }

    Ok(BakedVariableAddress(address.try_into().unwrap()))
}

#[derive(Deref, IntoIterator)]
pub(crate) struct BakedVariableAddress(VariableAddress);

#[derive(Debug, Clone)]
struct Stack(Box<[Value]>);

impl Stack {
    fn new(size: usize) -> Self {
        Self(vec![Value::Null; size].into_boxed_slice())
    }

    fn get(&self, index: usize) -> Result<&Value> {
        Ok(&self.0[index])
    }

    fn get_mut(&mut self, index: usize) -> Result<&mut Value> {
        Ok(&mut self.0[index])
    }

    fn set(&mut self, index: usize, new_value: Value) -> Result<()> {
        self.0[index] = new_value;
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    stack: Stack,
}

impl Scope {
    pub fn new(size: usize) -> Self {
        Self {
            stack: Stack::new(size),
        }
    }

    pub fn set(&mut self, index: usize, value: Value) {
        let _ = self.stack.set(index, value);
    }

    pub(crate) fn query_variable(
        &self,
        address: BakedVariableAddress,
        contained_module_id: &String,
    ) -> Result<Value> {
        let mut address = address.into_iter();

        let first_addressant = address.next().unwrap();

        let first_identifier = match first_addressant {
            VariableAddressant::StackIndex(idx) => idx,
            VariableAddressant::Identifier(_) |
            VariableAddressant::Index(_) => {
                panic!("Unsupported scope address!");
            }
            VariableAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };

        let value = self.stack
            .get(first_identifier)?;
        
        value::get(value, address, contained_module_id)
    }

    pub(crate) fn set_variable(
        &mut self,
        address: BakedVariableAddress,
        contained_module_id: &String,
        value: Value,
    ) -> Result<()> {
        let mut address = address.into_iter();

        let first_addressant = address.next().unwrap();

        let first_identifier = match first_addressant {
            VariableAddressant::StackIndex(idx) => idx,
            VariableAddressant::Identifier(_) |
            VariableAddressant::Index(_) => {
                panic!("Unsupported scope address!");
            }
            VariableAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };

        let value_ref = self.stack
            .get_mut(first_identifier)?;
        
        value::set(value_ref, address, contained_module_id, value)
    }

    pub(crate) fn reference_variable(
        &self,
        address: BakedVariableAddress,
        contained_module_id: &String,
    ) -> Result<Value> {
        let mut address = address.into_iter();

        let first_addressant = address.next().unwrap();

        let first_identifier = match first_addressant {
            VariableAddressant::StackIndex(idx) => idx,
            VariableAddressant::Identifier(_) |
            VariableAddressant::Index(_) => {
                panic!("Unsupported scope address!: {:?} {:?}", first_addressant, address);
            }
            VariableAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };

        let value = self.stack
            .get(first_identifier)?;
        
        value::reference(value, address, contained_module_id)
    }

    pub(crate) fn clone_variable(
        &self,
        address: BakedVariableAddress,
        contained_module_id: &String,
    ) -> Result<Value> {
        let mut address = address.into_iter();

        let first_addressant = address.next().unwrap();

        let first_identifier = match first_addressant {
            VariableAddressant::StackIndex(idx) => idx,
            VariableAddressant::Identifier(_) | 
            VariableAddressant::Index(_) => {
                panic!("Unsupported scope address!");
            }
            VariableAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };

        let value = self.stack
            .get(first_identifier)?;
        
        value::clone_member(value, address, contained_module_id)
    }

    pub(crate) fn query_type(
        &self,
        address: BakedVariableAddress,
        contained_module_id: &String,
    ) -> Result<Value> {
        let mut address = address.into_iter();

        let first_addressant = address.next().unwrap();

        let first_identifier = match first_addressant {
            VariableAddressant::StackIndex(idx) => idx,
            VariableAddressant::Identifier(_) |
            VariableAddressant::Index(_) => {
                panic!("Unsupported scope address!");
            }
            VariableAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };

        let value = self.stack
            .get(first_identifier)?;
        
        value::get_type(value, address, contained_module_id)
    }
}
