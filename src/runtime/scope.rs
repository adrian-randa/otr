use std::{collections::HashMap};

use derive_more::{Deref, IntoIterator};

use crate::{
    core::{expression::variable::{VariableAddress, VariableAddressant}, value::Value}, error::runtime_error::RuntimeError, runtime::{environment::Environment, expressions::eval_expression, scope::vec_map::VecMap, value}
};

use crate::error::Result;



pub(crate) fn try_bake_variable_address(address: VariableAddress, environment: &Environment) -> Result<BakedVariableAddress> {
    let mut out = Vec::with_capacity(address.len());

    for addressant in address {
        let addressant = match addressant {
            VariableAddressant::Identifier(ident) => VariableAddressant::Identifier(ident),
            VariableAddressant::Index(idx) => VariableAddressant::Index(idx),
            VariableAddressant::DynamicIndex(expression) => {
                let value = eval_expression(&expression, environment)?;
                let idx: usize = match value {
                    Value::Integer(value) => {
                        let idx = value.try_into().unwrap();

                        idx
                    }
                    _ => {
                        return Err(RuntimeError::TypeMismatch {
                            expected: crate::core::r#type::Type::Integer,
                            found: value.get_type_id(),
                        }
                        .boxed())
                    }
                };

                VariableAddressant::Index(idx)
            }
        };

        out.push(addressant);
    }

    Ok(BakedVariableAddress(out.try_into().unwrap()))
}

#[derive(Deref, IntoIterator)]
pub(crate) struct BakedVariableAddress(VariableAddress);

pub mod vec_map;

#[derive(Debug, Clone)]
struct Stack(Vec<VecMap<String, Value>>);

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    fn new() -> Self {
        Self(vec![VecMap::new()])
    }

    fn from_members(members: impl IntoIterator<Item = (String, Value)>) -> Self {
        let mut map = VecMap::new();

        for (key, value) in members {
            map.insert(key, value);
        }

        Self(vec![map])
    }

    fn insert_members(&mut self, members: impl IntoIterator<Item = (String, Value)>) {
        let last = self.0.len() - 1;
        let last = &mut  self.0[last];
        
        for (key, value) in members {
            last.insert(key, value);
        }
    }

    fn grow(&mut self) {
        self.0.push(VecMap::new());
    }

    fn shrink(&mut self) {
        self.0.pop();
    }

    fn push(&mut self, identifier: String, value: Value) -> Result<()> {
        let last = self.0.len() - 1;
        if self.0[last].insert(identifier.clone(), value).is_some() {
            return Err(RuntimeError::VariableAlreadyPresent {
                variable_identifier: identifier,
            }
            .boxed());
        }

        Ok(())
    }

    fn pop(&mut self, identifier: &String) -> Result<()> {
        let last = self.0.len() - 1;
        if self.0[last].remove(identifier).is_none() {
            return Err(RuntimeError::NoSuchVariable {
                variable_identifier: identifier.clone(),
            }
            .boxed());
        }

        Ok(())
    }

    fn get(&self, identifier: &String) -> Result<&Value> {
        for i in (0..self.0.len()).rev() {
            if let Some(value) = self.0[i].get(identifier) {
                return Ok(value);
            }
        }

        Err(RuntimeError::NoSuchVariable {
            variable_identifier: identifier.clone(),
        }
        .boxed())
    }

    fn get_mut(&mut self, identifier: &String) -> Result<&mut Value> {
        let last = self.0.len() - 1;

        let mut idx = None;

        for i in (0..=last).rev() {
            if self.0[i].contains_key(identifier) {
                idx = Some(i);
                break;
            }
        }

        if let Some(i) = idx {
            return Ok(self.0[i].get_mut(identifier).unwrap());
        }
        Err(RuntimeError::NoSuchVariable {
            variable_identifier: identifier.clone(),
        }
        .boxed())
    }

    fn _set(&mut self, identifier: &String, new_value: Value) -> Result<()> {
        for i in (0..self.0.len()).rev() {
            if let Some(value) = self.0[i].get_mut(identifier) {
                *value = new_value;
                return Ok(());
            }
        }

        Err(RuntimeError::NoSuchVariable {
            variable_identifier: identifier.clone(),
        }
        .boxed())
    }
}

#[derive(Debug, Default, Clone)]
pub struct Scope {
    stack: Stack,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
        }
    }

    pub fn from_members(members: HashMap<String, Value>) -> Self {
        Self {
            stack: Stack::from_members(members),
        }
    }

    pub fn insert_members(&mut self, members: HashMap<String, Value>) {
        self.stack.insert_members(members);
    }

    pub fn push(&mut self, identifier: String) -> Result<()> {
        self.stack.push(identifier, Value::Null)
    }

    pub fn pop(&mut self, identifier: &String) -> Result<()> {
        self.stack.pop(&identifier)
    }

    pub fn grow_stack(&mut self) {
        self.stack.grow();
    }

    pub fn shrink_stack(&mut self) {
        self.stack.shrink();
    }

    pub(crate) fn query_variable(
        &self,
        address: BakedVariableAddress,
        contained_module_id: &String,
    ) -> Result<Value> {
        let mut address = address.into_iter();

        let first_addressant = address.next().unwrap();

        let first_identifier = match first_addressant {
            VariableAddressant::Identifier(ident) => ident,
            VariableAddressant::Index(_) => {
                panic!("Unsupported scope address!");
            }
            VariableAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };

        let value = self.stack
            .get(&first_identifier)?;
        
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
            VariableAddressant::Identifier(ident) => ident,
            VariableAddressant::Index(_) => {
                panic!("Unsupported scope address!");
            }
            VariableAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };

        let value_ref = self.stack
            .get_mut(&first_identifier)?;
        
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
            VariableAddressant::Identifier(ident) => ident,
            VariableAddressant::Index(_) => {
                panic!("Unsupported scope address!");
            }
            VariableAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };

        let value =self.stack
            .get(&first_identifier)?;
        
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
            VariableAddressant::Identifier(ident) => ident,
            VariableAddressant::Index(_) => {
                panic!("Unsupported scope address!");
            }
            VariableAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };

        let value = self.stack
            .get(&first_identifier)?;
        
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
            VariableAddressant::Identifier(ident) => ident,
            VariableAddressant::Index(_) => {
                panic!("Unsupported scope address!");
            }
            VariableAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };

        let value = self.stack
            .get(&first_identifier)?;
        
        value::get_type(value, address, contained_module_id)
    }
}
