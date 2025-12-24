use std::cell::{Cell, RefCell};
use std::fmt::Display;
use std::ops::Deref;
use std::vec::IntoIter;
use std::{collections::HashMap, rc::Rc};

use derive_more::{Deref, IntoIterator};
use num::traits::identities;

use crate::compiler::CompilerError;
use crate::compiler::expression_parser::ExpressionParser;
use crate::lexer::token::{LiteralToken, ParenthesisType, PunctuationToken, Token};
use crate::runtime::environment::Environment;
use crate::runtime::expressions::ProcedureCallExpression;
use crate::runtime::procedures::{CompiledProcedure, Procedure};

pub mod environment;
pub mod expressions;
pub mod module;
pub mod procedures;

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
}

pub trait Expression: std::fmt::Debug {
    fn eval(&self, environment: &Environment) -> Result<Value, RuntimeError>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Array(Vec<Value>),
    Struct(Struct),
}

impl TryFrom<LiteralToken> for Value {
    type Error = CompilerError;

    fn try_from(value: LiteralToken) -> Result<Self, Self::Error> {
        match value {
            LiteralToken::WholeNumber(num) => {
                Ok(Self::Integer(
                    num.parse().map_err(|_| CompilerError {
                        message: format!("Could not parse '{}' as a whole number!", num)
                    })?
                ))
            },
            LiteralToken::Decimal(num) => {
                Ok(Self::Float(
                    num.parse().map_err(|_| CompilerError {
                        message: format!("Could not parse '{}' as a decimal number!", num)
                    })?
                ))
            },
            LiteralToken::Boolean(b) => {
                match &b as &str {
                    "true" => Ok(Self::Bool(true)),
                    "false" => Ok(Self::Bool(false)),
                    _ => Err(CompilerError { message: format!("Could not parse {} as a boolean!", b) })
                }
            },
            LiteralToken::Char(c) => {
                Ok(Self::Char(c.chars().next().ok_or(CompilerError {
                    message: format!("Could not parse {} as a char!", c)
                })?))
            },
            LiteralToken::String(str) => {
                Ok(Self::String(str))
            },
        }
    }
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
            Value::Struct(object) => object.get_struct_id().to_string(),
        }
    }
}

impl Expression for Value {
    fn eval(&self, _environment: &Environment) -> Result<Value, RuntimeError> {
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Member {
    is_private: bool,
    value: Value,
}

impl From<(bool, Value)> for Member {
    fn from((is_private, value): (bool, Value)) -> Self {
        Self { is_private, value }
    }
}

impl Member {
    pub fn get_value(&self) -> &Value {
        &self.value
    }

    pub fn get_value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    pub fn get_value_if_public(&self) -> Result<&Value, RuntimeError> {
        if self.is_private {
            Err(RuntimeError {
                message: "Tried to access a private field!".into(),
            })
        } else {
            Ok(&self.value)
        }
    }

    pub fn get_value_mut_if_public(&mut self) -> Result<&mut Value, RuntimeError> {
        if self.is_private {
            Err(RuntimeError {
                message: "Tried to access a private field!".into(),
            })
        } else {
            Ok(&mut self.value)
        }
    }

    pub fn set_value(&mut self, value: Value) {
        self.value = value;
    }

    pub fn set_if_public(&mut self, value: Value) -> Result<(), RuntimeError> {
        if self.is_private {
            Err(RuntimeError {
                message: "Tried to access a private field!".into(),
            })
        } else {
            self.value = value;
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberMap {
    members: HashMap<String, Member>,
}

impl MemberMap {
    pub fn get_member(&self, ident: &String) -> Result<&Value, RuntimeError> {
        let member = self.members.get(ident).ok_or(RuntimeError {
            message: format!("No member labeled '{}'!", ident),
        })?;

        Ok(member.get_value())
    }

    pub fn get_member_mut(&mut self, ident: &String) -> Result<&mut Value, RuntimeError> {
        let member = self.members.get_mut(ident).ok_or(RuntimeError {
            message: format!("No member labeled '{}'!", ident),
        })?;

        Ok(member.get_value_mut())
    }

    pub fn get_public_member(&self, ident: &String) -> Result<&Value, RuntimeError> {
        let member = self.members.get(ident).ok_or(RuntimeError {
            message: format!("No member labeled '{}'!", ident),
        })?;

        member.get_value_if_public()
    }

    pub fn get_public_member_mut(&mut self, ident: &String) -> Result<&mut Value, RuntimeError> {
        let member = self.members.get_mut(ident).ok_or(RuntimeError {
            message: format!("No member labeled '{}'!", ident),
        })?;

        member.get_value_mut_if_public()
    }

    pub fn set_member(&mut self, ident: &String, value: Value) -> Result<(), RuntimeError> {
        let member = self.members.get_mut(ident).ok_or(RuntimeError {
            message: format!("No member labeled '{}'!", ident),
        })?;

        member.set_if_public(value)
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleAddress {
    module_id: String,
    identifier: String,
}

impl From<(&str, &str)> for ModuleAddress {
    fn from(value: (&str, &str)) -> Self {
        Self {
            module_id: value.0.to_string(),
            identifier: value.1.to_string(),
        }
    }
}

impl Display for ModuleAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.module_id, self.identifier)
    }
}

impl ModuleAddress {
    pub fn new(module_id: String, identifier: String) -> Self {
        Self {
            module_id,
            identifier,
        }
    }

    pub fn get_module_id(&self) -> &String {
        &self.module_id
    }

    pub fn get_identifier(&self) -> &String {
        &self.identifier
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    //TODO: Remove public visibility
    pub struct_id: ModuleAddress,
    pub members: MemberMap,
}

impl Struct {
    pub fn get_struct_id(&self) -> &ModuleAddress {
        &self.struct_id
    }

    pub fn get_members(&self) -> &MemberMap {
        &self.members
    }

    pub fn get_members_mut(&mut self) -> &mut MemberMap {
        &mut self.members
    }
}

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
struct BakedScopeAddress(Vec<ScopeAddressant>);

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
                "Could not find the variable \"{}\" in this scope!",
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
                "Could not find the variable \"{}\" in this scope!",
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
                "Could not find the variable \"{}\" in this scope!",
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

    fn get_variable(
        &self,
        address: BakedScopeAddress,
        contained_module_id: &String,
    ) -> Result<&Value, RuntimeError> {
        let mut addressants = address.into_iter();

        let first_addressant = addressants.next().unwrap();

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

        let mut value = self.stack.get(&first_identifier)?;

        for subaddressant in addressants {
            match subaddressant {
                ScopeAddressant::Identifier(ident) => {
                    if let Value::Struct(ref obj) = value {
                        if obj.get_struct_id().get_module_id() != contained_module_id {
                            Err(RuntimeError {
                                message: format!(
                                    "Tried to access field \"{}\" of {} outside it's module!",
                                    ident,
                                    obj.get_struct_id()
                                ),
                            })?;
                        }

                        value = obj.get_members().get_public_member(&ident)?;
                    } else {
                        Err(RuntimeError {
                            message: format!(
                                "This variable does not have a member labeled \"{}\"!",
                                ident
                            ),
                        })?;
                    }
                }
                ScopeAddressant::Index(idx) => {
                    if let Value::Array(ref arr) = value {
                        let new_value = arr.get(idx).ok_or(RuntimeError {
                            message: format!(
                                "Index out of bounds: index was {}, array length was {}!",
                                idx,
                                arr.len()
                            ),
                        })?;

                        value = new_value;
                    } else {
                        Err(RuntimeError {
                            message: "This value can not be indexed!".into(),
                        })?;
                    }
                }
                ScopeAddressant::DynamicIndex(_) => {
                    panic!("Found dynamic index as addressant after baking!");
                }
            }
        }

        Ok(value)
    }

    fn get_variable_mut(
        &mut self,
        address: BakedScopeAddress,
        contained_module_id: &String,
    ) -> Result<&mut Value, RuntimeError> {
        let mut addressants = address.into_iter();

        let first_addressant = addressants.next().unwrap();

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

        let mut value = self
            .stack
            .get_mut(&first_identifier)?;

        for subaddressant in addressants {
            match subaddressant {
                ScopeAddressant::Identifier(ident) => {
                    if let Value::Struct(ref mut s) = value {
                        if s.get_struct_id().get_module_id() != contained_module_id {
                            Err(RuntimeError {
                                message: format!(
                                    "Tried to access field '{}' of {} outside it's module!",
                                    ident,
                                    s.get_struct_id()
                                ),
                            })?;
                        }

                        value = s.get_members_mut().get_public_member_mut(&ident)?;
                    } else {
                        Err(RuntimeError {
                            message: format!(
                                "This variable does not have a member labeled '{}'!",
                                ident
                            ),
                        })?;
                    }
                }
                ScopeAddressant::Index(idx) => {
                    if let Value::Array(ref mut arr) = value {
                        let array_length = arr.len();

                        let new_value = arr.get_mut(idx).ok_or(RuntimeError {
                            message: format!(
                                "Index out of bounds: index was {}, array length was {}!",
                                idx, array_length
                            ),
                        })?;

                        value = new_value;
                    } else {
                        Err(RuntimeError {
                            message: "This value can not be indexed!".into(),
                        })?;
                    }
                }
                ScopeAddressant::DynamicIndex(_) => {
                    panic!("Found dynamic index as addressant after baking!");
                }
            }
        }

        Ok(value)
    }
}


#[derive(Debug)]
pub struct RuntimeObject {
    pub(crate) base_environement: Environment,
    pub(crate) entrypoint: Option<ModuleAddress>
}

impl RuntimeObject {
    pub(crate) fn new() -> Self {
        Self {
            base_environement: Environment::new("".into()),
            entrypoint: None,
        }
    }

    pub fn execute(self) -> Result<Value, RuntimeError> {
        let entrypoint = self.entrypoint.ok_or(RuntimeError {
            message: "No specified entrypoint!".into()
        })?;

        let main_expression = ProcedureCallExpression::new(
            entrypoint,
            Vec::new()
        );

        main_expression.eval(&self.base_environement)
    }
}