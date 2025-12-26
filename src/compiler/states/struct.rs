use crate::{compiler::{CompilerError, CompilerState, states::module::CompilerModuleState}, lexer::token::{KeywordToken, ParenthesisType, PunctuationToken, Token}, runtime::{ModuleAddress, Struct, Value}};

enum CompilerStructSubstate {
    Identifier,
    PreFields,
    Field {
        is_public: bool,
    },
    AfterField,
}

pub struct CompilerStructState {
    module: CompilerModuleState,
    substate: CompilerStructSubstate,

    identifier: Option<String>,
    fields: Vec<(String, bool)>,
}

impl CompilerState for CompilerStructState {
    fn read(mut self: Box<Self>, token: crate::lexer::token::Token, compiler_environment: &mut crate::compiler::CompilerEnvironment) -> Result<Box<dyn CompilerState>, crate::compiler::CompilerError> {
        match self.substate {
            CompilerStructSubstate::Identifier => {
                match token {
                    Token::Identifier(ident) => {
                        self.identifier = Some(ident);
                        self.substate = CompilerStructSubstate::PreFields;
                        return Ok(self)
                    }

                    other => {
                        return Err(CompilerError {
                            message: format!("Unexpected token. Expected identifier, found {:?}!", other)
                        });
                    }
                }
            },
            CompilerStructSubstate::PreFields => {
                match token {
                    Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) => {
                        self.substate = CompilerStructSubstate::Field {
                            is_public: false
                        };
                        return Ok(self);
                    }

                    other => {
                        return Err(CompilerError {
                            message: format!("Unexpected token. Expected '{{', found {:?}!", other)
                        });
                    }
                }
            },
            CompilerStructSubstate::Field { is_public } => {
                match token {
                    Token::Keyword(KeywordToken::Public) => {
                        self.substate = CompilerStructSubstate::Field { is_public: true };
                        Ok(self)
                    }

                    Token::Identifier(ident) => {
                        self.fields.push((ident, is_public));
                        self.substate = CompilerStructSubstate::AfterField;
                        return Ok(self);
                    }
                    
                    other => {
                        return Err(CompilerError {
                            message: format!("Unexpected token. Expected identifier, found {:?}!", other)
                        });
                    }
                }
            },
            CompilerStructSubstate::AfterField => {
                match token {
                    Token::Punctuation(PunctuationToken::Comma) => {
                        self.substate = CompilerStructSubstate::Field {
                            is_public: false,
                        };
                        return Ok(self);
                    }

                    Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Closing)) => {
                        let struct_id = ModuleAddress::new(
                            self.module.get_name().unwrap().to_owned(),
                            self.identifier.clone().unwrap()
                        );

                        let mut prototype = Struct::new(struct_id);

                        let members = prototype.get_members_mut();

                        for field in self.fields {
                            members.insert_member(field.0, Value::Null, field.1).map_err(|err| CompilerError {
                                message: format!("Error while parsing struct prototype: {:?}", err)
                            })?;
                        }

                        self.module.get_module_mut().insert_struct(self.identifier.unwrap(), prototype, false);

                        return Ok(Box::new(self.module));
                    }

                    other => {
                        return Err(CompilerError {
                            message: format!("Unexpected token. Expected ',' or '}}', found {:?}!", other)
                        });
                    }
                }
            }
        }
    }

    fn finalize(self: Box<Self>) -> Result<crate::runtime::environment::Environment, crate::compiler::CompilerError> {
        Err(CompilerError {
            message: "Unfinished module declaration!".into()
        })
    }
}

impl CompilerStructState {
    pub fn new(module: CompilerModuleState) -> Self {
        Self {
            module,
            substate: CompilerStructSubstate::Identifier,
            identifier: None,
            fields: Vec::new(),
        }
    }
}