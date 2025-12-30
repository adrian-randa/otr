use std::{collections::HashMap, ops::Deref, rc::Rc};

use derive_more::{Deref, IntoIterator};

use crate::{compiler::{CompilerError, expression_parser::ExpressionParser}, lexer::token::{ParenthesisType, PunctuationToken, Token}, runtime::{Expression, RuntimeError, Value, environment::Environment}};


#[derive(Debug, Clone)]
pub enum ScopeAddressant {
    Identifier(String),
    Index(usize),
    DynamicIndex(Rc<dyn Expression>),
}

impl From<&str> for ScopeAddressant {
    fn from(value: &str) -> Self {
        Self::Identifier(value.into())
    }
}

impl From<usize> for ScopeAddressant {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}

impl<E: Expression + 'static> From<E> for ScopeAddressant {
    fn from(value: E) -> Self {
        Self::DynamicIndex(Rc::new(value))
    }
}

#[derive(Debug, Clone)]
pub struct ScopeAddress(Vec<ScopeAddressant>);

impl TryFrom<Vec<ScopeAddressant>> for ScopeAddress {
    type Error = ();

    fn try_from(value: Vec<ScopeAddressant>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(())
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<Vec<Token>> for ScopeAddress {
    type Error = CompilerError;

    fn try_from(value: Vec<Token>) -> Result<Self, Self::Error> {
        let mut tokens = value.into_iter();
        
        let mut addressants = Vec::new();

        while let Some(token) = tokens.next() {
            match token {
                Token::Identifier(ident) => {
                    addressants.push(ScopeAddressant::Identifier(ident));
                }
                Token::Punctuation(PunctuationToken::Dot) => {}
                Token::Punctuation(PunctuationToken::SquareBrackets(ParenthesisType::Opening)) => {
                    let index_expression = ExpressionParser::take_until_closing(
                        &mut tokens,
                        Token::Punctuation(PunctuationToken::SquareBrackets(ParenthesisType::Closing))
                    )?;

                    let index_expression = ExpressionParser::parse(index_expression)?;

                    addressants.push(ScopeAddressant::DynamicIndex(index_expression.into()));
                }

                other => {
                    return Err(CompilerError {
                        message: format!("Invalid address. Found unexpected token {:?}!", other)
                    });
                }
            }
        }


        addressants.try_into().map_err(|_| CompilerError { message: "Address could not be parsed!".into() })
    }
}

impl ScopeAddress {
    pub(crate) fn try_bake(self, environment: &Environment) -> Result<BakedScopeAddress, RuntimeError> {
        let mut out = Vec::with_capacity(self.0.len());

        for addressant in self.0 {
            let addressant = match addressant {
                ScopeAddressant::Identifier(ident) => ScopeAddressant::Identifier(ident),
                ScopeAddressant::Index(idx) => ScopeAddressant::Index(idx),
                ScopeAddressant::DynamicIndex(expression) => {
                    let value = expression.eval(environment)?;
                    let idx: usize = match value {
                        Value::Integer(value) => {
                            let idx =
                                value.try_into().map_err(|err: std::num::TryFromIntError| {
                                    RuntimeError {
                                        message: err.to_string(),
                                    }
                                })?;

                            idx
                        }
                        _ => {
                            return Err(RuntimeError {
                                message: format!(
                                    "Mismatched types! Expected Integer, found {}!",
                                    value.get_type_id()
                                ),
                            })
                        }
                    };

                    ScopeAddressant::Index(idx)
                }
            };

            out.push(addressant);
        }

        Ok(BakedScopeAddress(out))
    }
}

#[derive(Deref, IntoIterator)]
pub(crate) struct BakedScopeAddress(Vec<ScopeAddressant>);

#[derive(Debug, Clone)]
struct Stack (Vec<HashMap<String, Value>>);

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    fn new() -> Self {
        Self(vec![HashMap::new()])    
    }

    fn from_members(members: HashMap<String, Value>) -> Self {
        Self(vec![members])
    }

    fn insert_members(&mut self, members: HashMap<String, Value>) {
        let last = self.0.len() - 1;
        self.0[last].extend(members.into_iter());
    }
    
    fn grow(&mut self) {
        self.0.push(HashMap::new());
    }

    fn shrink(&mut self) {
        self.0.pop();
    }

    fn push(&mut self, identifier: String, value: Value) -> Result<(), RuntimeError> {
        let last = self.0.len() - 1;
        if self.0[last].insert(identifier.clone(), value).is_some() {
            return Err(RuntimeError {
                message: format!("Variable '{}' already present in this scope!", identifier)
            });
        }

        Ok(())
    }

    fn pop(&mut self, identifier: &String) -> Result<(), RuntimeError> {
        let last = self.0.len() - 1;
        if self.0[last].remove(identifier).is_none() {
            return Err(RuntimeError {
                message: format!("Variable '{}' cannot be popped from the stack as it is not present!", identifier)
            });
        }

        Ok(())
    }

    fn get(&self, identifier: &String) -> Result<&Value, RuntimeError> {
        for i in (0..self.0.len()).rev() {
            if let Some(value) = self.0[i].get(identifier) {
                return Ok(value);
            }
        }

        Err(RuntimeError {
            message: format!(
                "Could not find the variable '{}' in this scope!",
                identifier
            ),
        })
    }

    fn get_mut(&mut self, identifier: &String) -> Result<&mut Value, RuntimeError> {
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
        Err(RuntimeError {
            message: format!(
                "Could not find the variable '{}' in this scope!",
                identifier
            ),
        })
    }

    fn set(&mut self, identifier: &String, new_value: Value) -> Result<(), RuntimeError> {
        for i in (0..self.0.len()).rev() {
            if let Some(value) = self.0[i].get_mut(identifier) {
                *value = new_value;
                return Ok(());
            }
        }

        Err(RuntimeError {
            message: format!(
                "Could not find the variable '{}' in this scope!",
                identifier
            ),
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct Scope {
    //TODO: Remove public visibility
    pub stack: Stack,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
        }
    }

    pub fn from_members(members: HashMap<String, Value>) -> Self {
        Self { stack: Stack::from_members(members) }
    }

    pub fn insert_members(&mut self, members: HashMap<String, Value>) {
        self.stack.insert_members(members);
    }

    pub fn push(&mut self, identifier: String) -> Result<(), RuntimeError> {
        self.stack.push(identifier, Value::Null)
    }

    pub fn pop(&mut self, identifier: &String) -> Result<(), RuntimeError> {
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
        address: BakedScopeAddress,
        contained_module_id: &String,
    ) -> Result<Value, RuntimeError> {
        let mut address = address.into_iter();

        let first_addressant = address.next().unwrap();

        let first_identifier = match first_addressant {
            ScopeAddressant::Identifier(ident) => ident,
            ScopeAddressant::Index(_) => {
                return Err(RuntimeError {
                    message: "Expected variable identifier, found index!".into(),
                })
            }
            ScopeAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };

        self.stack.get(&first_identifier)?.query(address, contained_module_id)
    }

    pub(crate) fn set_variable(&mut self, address: BakedScopeAddress, contained_module_id: &String, value: Value) -> Result<(), RuntimeError> {
        let mut address = address.into_iter();

        let first_addressant = address.next().unwrap();

        let first_identifier = match first_addressant {
            ScopeAddressant::Identifier(ident) => ident,
            ScopeAddressant::Index(_) => {
                return Err(RuntimeError {
                    message: "Expected variable identifier, found index!".into(),
                })
            }
            ScopeAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };

        self.stack.get_mut(&first_identifier)?.set(address, contained_module_id, value)
    }

    pub(crate) fn reference_variable(&self, address: BakedScopeAddress, contained_module_id: &String) -> Result<Value, RuntimeError> {
        let mut address = address.into_iter();

        let first_addressant = address.next().unwrap();

        let first_identifier = match first_addressant {
            ScopeAddressant::Identifier(ident) => ident,
            ScopeAddressant::Index(_) => {
                return Err(RuntimeError {
                    message: "Expected variable identifier, found index!".into(),
                })
            }
            ScopeAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };

        self.stack.get(&first_identifier)?.reference(address, contained_module_id)
    }

    pub(crate) fn clone_variable(&self, address: BakedScopeAddress, contained_module_id: &String) -> Result<Value, RuntimeError> {
        let mut address = address.into_iter();

        let first_addressant = address.next().unwrap();

        let first_identifier = match first_addressant {
            ScopeAddressant::Identifier(ident) => ident,
            ScopeAddressant::Index(_) => {
                return Err(RuntimeError {
                    message: "Expected variable identifier, found index!".into(),
                })
            }
            ScopeAddressant::DynamicIndex(_) => {
                panic!("Found dynamic index as addressant after baking!");
            }
        };

        self.stack.get(&first_identifier)?.clone_variable(address, contained_module_id)
    }
}
