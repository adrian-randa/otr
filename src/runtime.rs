use std::cell::{Cell, RefCell};
use std::vec::IntoIter;
use std::{collections::HashMap, rc::Rc};

use derive_more::Deref;
use num::traits::identities;

use crate::runtime::expressions::{Expression};
use crate::runtime::procedures::{CompiledProcedure, Procedure};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Array(Vec<Value>),
    Object(Object)
}

impl Value {

    pub fn get_type_id(&self) -> String {
        match self {
            Value::Null => "Null".into(),
            Value::Integer(_) => "Integer".into(),
            Value::Float(_) => "Float".into(),
            Value::String(_) => "String".into(),
            Value::Char(_) => "Char".into(),
            Value::Bool(_) => "Bool".into(),
            Value::Array(_) => "Array".into(),
            Value::Object(object) => object.class_id.clone()
        }
    }

}

impl Expression for Value {
    fn eval(&self, _environment: &Environment) -> Result<Value, RuntimeError> {
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object { //TODO: Remove public visibility
    pub class_id: String,
    pub members: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
}

pub mod expressions;
pub mod procedures;


#[derive(Clone)]
pub enum ScopeAddressant {
    Identifier(String),
    Index(usize),
    DynamicIndex(Rc<dyn Expression>),
}

#[derive(Clone, Deref)]
pub struct ScopeAddress(Vec<ScopeAddressant>);

impl From<Vec<ScopeAddressant>> for ScopeAddress {
    fn from(value: Vec<ScopeAddressant>) -> Self {
        Self(value)
    }
}

impl ScopeAddress {

    pub fn new(address: Vec<ScopeAddressant>) -> Self {
        Self(address)
    }

    fn try_bake(self, environment: &Environment) -> Result<BakedScopeAddress, RuntimeError> {
        let mut out = Vec::with_capacity(self.0.len());

        for addressant in self.0 {
            let addressant = match addressant {
                ScopeAddressant::Identifier(ident) => ScopeAddressant::Identifier(ident),
                ScopeAddressant::Index(idx) => ScopeAddressant::Index(idx),
                ScopeAddressant::DynamicIndex(expression) => {
                    let value = expression.eval(environment)?;
                    let idx: usize = match value {
                        Value::Integer(value) => {
                            let idx = value.try_into()
                                .map_err(|err: std::num::TryFromIntError| RuntimeError {
                                    message: err.to_string()
                                })?;
                            
                            idx
                        },
                        _ => {
                            return Err(RuntimeError {
                                message: format!("Mismatched types! Expected Integer, found {}!", value.get_type_id())
                            })
                        }
                    };
                    
                    ScopeAddressant::Index(idx)
                },
            };

            out.push(addressant);
        }

        Ok(BakedScopeAddress(out))
    }

}

#[derive(Deref)]
struct BakedScopeAddress(Vec<ScopeAddressant>);

impl BakedScopeAddress {

    fn unpack(self) -> Vec<ScopeAddressant> {
        self.0
    }

}

#[derive(Debug)]
pub struct Scope { //TODO: Remove public visibility
    pub members: HashMap<String, Value>,
}

impl Scope {

    pub fn new() -> Self {
        Self { members: HashMap::new() }
    }

    pub fn from_members(members: HashMap<String, Value>) -> Self {
        Self { members }
    }

    pub fn push(&mut self, identifier: String) {
        self.members.insert(identifier, Value::Null);
    }

    pub fn pop(&mut self, identifier: &String) {
        self.members.remove(identifier);
    }

    fn get_variable(&self, address: BakedScopeAddress) -> Result<&Value, RuntimeError> {
        let mut addressants = address.unpack().into_iter();

        let first_addressant = addressants.next()
            .ok_or(RuntimeError{
                message: "Tried looking up a variable in scope without an address!".into()
            })?;
        
        let first_identifier = match first_addressant {
            ScopeAddressant::Identifier(ident) => ident,
            ScopeAddressant::Index(_) => return Err(RuntimeError{
                message: "Expected variable identifier, found index!".into()
            }),
            ScopeAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };
        
        let mut value = self.members.get(&first_identifier)
            .ok_or(RuntimeError{
                message: format!("Could not find the variable \"{}\" in this scope!", first_identifier)
            })?;

        for subaddressant in addressants {
            match subaddressant {
                ScopeAddressant::Identifier(ident) => {
                    if let Value::Object(ref obj) = value {
                        let new_value = obj
                            .members
                            .get(&ident)
                            .ok_or(RuntimeError {
                                message: format!("Object does not have a member labeled \"{}\"!", ident)
                            })?;

                        value = new_value;
                    } else {
                        Err(RuntimeError{
                            message: format!("This variable does not have a member labeled \"{}\"!", ident)
                        })?;
                    }
                },
                ScopeAddressant::Index(idx) => {
                    if let Value::Array(ref arr) = value {
                        let new_value = arr
                            .get(idx)
                            .ok_or(RuntimeError{
                                message: format!("Index out of bounds: index was {}, array length was {}!", idx, arr.len())
                            })?;

                        value = new_value;
                    } else {
                        Err(RuntimeError{
                            message: "This value can not be indexed!".into()
                        })?;
                    }
                },
                ScopeAddressant::DynamicIndex(_) => {
                    panic!("Found dynamic index as addressant after baking!");
                }
            }
        }

        Ok(value)
    }

    fn get_variable_mut(&mut self, address: BakedScopeAddress) -> Result<&mut Value, RuntimeError> {
        let mut addressants = address.unpack().into_iter();

        let first_addressant = addressants.next()
            .ok_or(RuntimeError{
                message: "Tried looking up a variable in scope without an address!".into()
            })?;
        
        let first_identifier = match first_addressant {
            ScopeAddressant::Identifier(ident) => ident,
            ScopeAddressant::Index(_) => return Err(RuntimeError{
                message: "Expected variable identifier, found index!".into()
            }),
            ScopeAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };
        
        let mut value = self.members.get_mut(&first_identifier)
            .ok_or(RuntimeError{
                message: format!("Could not find the variable \"{}\" in this scope!", first_identifier)
            })?;

        for subaddressant in addressants {
            match subaddressant {
                ScopeAddressant::Identifier(ident) => {
                    if let Value::Object(ref mut obj) = value {
                        let new_value = obj
                            .members
                            .get_mut(&ident)
                            .ok_or(RuntimeError {
                                message: format!("Object does not have a member labeled \"{}\"!", ident)
                            })?;

                        value = new_value;
                    } else {
                        Err(RuntimeError{
                            message: format!("This variable does not have a member labeled \"{}\"!", ident)
                        })?;
                    }
                },
                ScopeAddressant::Index(idx) => {
                    if let Value::Array(ref mut arr) = value {
                        let array_length = arr.len();

                        let new_value = arr
                            .get_mut(idx)
                            .ok_or(RuntimeError{
                                message: format!("Index out of bounds: index was {}, array length was {}!", idx, array_length)
                            })?;

                        value = new_value;
                    } else {
                        Err(RuntimeError{
                            message: "This value can not be indexed!".into()
                        })?;
                    }
                },
                ScopeAddressant::DynamicIndex(_) => {
                    panic!("Found dynamic index as addressant after baking!");
                }
            }
        }

        Ok(value)
    }

}

pub struct Environment { //TODO: Remove public visibility
    pub procedures: Rc<HashMap<String, Box<dyn Procedure>>>,
    pub scope: Scope,
}

impl Environment {
    pub fn new() -> Self {
        Self { procedures: Rc::new(HashMap::new()), scope: Scope::new() }
    }

    pub fn get_procedure_by_id(&self, id: &String) -> Result<&Box<dyn Procedure>, RuntimeError> {
        self.procedures.get(id).ok_or(RuntimeError { message: format!("Could not find procedure labeled \"{}\"", id) })
    }

    pub fn clone_with_scope(&self, new_scope: Scope) -> Self {
        Self {
            procedures: self.procedures.clone(),
            scope: new_scope,
        }
    }

    pub fn lookup_variable(&self, address: ScopeAddress) -> Result<Value, RuntimeError> {
        let address = address.try_bake(self)?;
        
        let value = self.scope.get_variable(address)?;

        Ok(value.clone())
    }

    
    pub fn set_variable(&mut self, address: ScopeAddress, new_value: Value) -> Result<(), RuntimeError> {
        let address = address.try_bake(self)?;

        let value = self.scope.get_variable_mut(address)?;

        *value = new_value;

        Ok(())
    }
}