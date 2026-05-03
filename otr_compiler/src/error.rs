use colored::Colorize;

use otr_core::{r#type::Type, value::Value, error::Error};
use crate::lexer::token::Token;

#[derive(Debug)]
pub(crate) enum CompilerError {
    LiteralParseError {
        ty: Type,
        literal: String,
    },
    NoSuchMember {
        member_identifier: String,
    },
    InvalidScopeAddress {
        unexpected_token: Option<Token>,
    },
    NoScopeToClose,
    UnexpectedToken {
        expected: Option<String>,
        found: Token,
    },
    InvalidParenthesisStructure,
    InvalidExpression {
        message: String,
    },
    InvalidDefinition {
        message: String,
    },

    Unknown {
        message: String,
    },
}

impl Error for CompilerError {
    fn to_value(&self) -> Value {
        panic!("Compiler Errors cannot be turned into values!")
    }
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            CompilerError::LiteralParseError { ty, literal } => {
                format!("Could not parse '{literal}' as {ty}!")
            }
            CompilerError::NoSuchMember { member_identifier } => {
                format!("No member labeled '{member_identifier}'!")
            }
            CompilerError::InvalidScopeAddress { unexpected_token } => format!(
                "Invalid scope address!{}",
                match unexpected_token {
                    Some(token) => format!(" Unexpected token: {:?}.", token),
                    None => "".to_string(),
                }
            ),
            CompilerError::NoScopeToClose => format!("There is no scope to close!"),
            CompilerError::UnexpectedToken { expected, found } => match expected {
                Some(expected) => {
                    format!("Unexpected token! Expected {expected}, found {:?}.", found)
                }
                None => format!("Unexpected token: {:?}!", found),
            },
            CompilerError::InvalidParenthesisStructure => format!("Invalid parenthesis structure!"),
            CompilerError::InvalidExpression { message } => {
                format!("Invalid expression! {message}")
            }
            CompilerError::InvalidDefinition { message } => {
                format!("Invalid definition! {message}")
            }
            CompilerError::Unknown { message } => format!("{message}"),
        };

        write!(
            f,
            "{} {}",
            "Compiler Error!".on_red(),
            (&message as &str).red()
        )
    }
}

impl CompilerError {
    pub(crate) fn boxed(self) -> Box<dyn Error> {
        Box::new(self)
    }
}

pub(crate) mod context;