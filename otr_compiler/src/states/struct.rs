use crate::{
    CompilerEnvironment, CompilerError, CompilerState, lexer::token::{KeywordToken, ParenthesisType, PunctuationToken, Token}, states::module::CompilerModuleState
};

use otr_core::{module::{CompiledModule, ModuleAddress}, r#struct::Struct, value::Value, error::Result};

enum CompilerStructSubstate {
    Identifier,
    PreFields,
    Field { is_public: bool },
    AfterField,
}

pub struct CompilerStructState {
    module: Box<CompilerModuleState>,
    substate: CompilerStructSubstate,

    identifier: Option<String>,
    fields: Vec<(String, bool)>,
}

impl CompilerState for CompilerStructState {
    fn read(
        mut self: Box<Self>,
        token: Token,
        compiler_environment: &mut CompilerEnvironment,
    ) -> Result<Box<dyn CompilerState>> {
        match self.substate {
            CompilerStructSubstate::Identifier => match token {
                Token::Identifier(ident) => {
                    self.identifier = Some(ident);
                    self.substate = CompilerStructSubstate::PreFields;
                    Ok(self)
                }

                other => {
                    Err(CompilerError::UnexpectedToken {
                        expected: Some("Identifier".into()),
                        found: other,
                    }
                    .boxed())
                }
            },
            CompilerStructSubstate::PreFields => match token {
                Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) => {
                    self.substate = CompilerStructSubstate::Field { is_public: false };
                    Ok(self)
                }

                other => {
                    Err(CompilerError::UnexpectedToken {
                        expected: Some("{".into()),
                        found: other,
                    }
                    .boxed())
                }
            },
            CompilerStructSubstate::Field { is_public } => match token {
                Token::Keyword(KeywordToken::Public) => {
                    self.substate = CompilerStructSubstate::Field { is_public: true };
                    Ok(self)
                }

                Token::Identifier(ident) => {
                    self.fields.push((ident, is_public));
                    self.substate = CompilerStructSubstate::AfterField;
                    Ok(self)
                }

                Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Closing)) => {
                    let struct_id = ModuleAddress::new(
                        self.module.get_name().unwrap().to_owned(),
                        self.identifier.clone().unwrap(),
                    );

                    let mut prototype = Struct::new(struct_id.clone());
                    compiler_environment.register_struct_ident(struct_id);

                    let members = prototype.get_members_mut();

                    for field in self.fields {
                        members.insert(field.0, Value::Null, field.1);
                    }

                    self.module.get_module_mut().insert_struct(
                        self.identifier.unwrap(),
                        prototype,
                        false,
                    );

                    Ok(self.module)
                }

                other => {
                    Err(CompilerError::UnexpectedToken {
                        expected: Some("Identifier".into()),
                        found: other,
                    }
                    .boxed())
                }
            },
            CompilerStructSubstate::AfterField => match token {
                Token::Punctuation(PunctuationToken::Comma) => {
                    self.substate = CompilerStructSubstate::Field { is_public: false };
                    Ok(self)
                }

                Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Closing)) => {
                    let struct_id = ModuleAddress::new(
                        self.module.get_name().unwrap().to_owned(),
                        self.identifier.clone().unwrap(),
                    );

                    let mut prototype = Struct::new(struct_id.clone());
                    compiler_environment.register_struct_ident(struct_id);

                    let members = prototype.get_members_mut();

                    for field in self.fields {
                        members.insert(field.0, Value::Null, field.1);
                    }

                    self.module.get_module_mut().insert_struct(
                        self.identifier.unwrap(),
                        prototype,
                        false,
                    );

                    Ok(self.module)
                }

                other => {
                    Err(CompilerError::UnexpectedToken {
                        expected: None,
                        found: other,
                    }
                    .boxed())
                }
            },
        }
    }

    fn finalize(self: Box<Self>) -> Result<CompiledModule> {
        Err(CompilerError::Unknown {
            message: "Unfinished module declaration!".into(),
        }
        .boxed())
    }
}

impl CompilerStructState {
    pub fn new(module: Box<CompilerModuleState>) -> Self {
        Self {
            module,
            substate: CompilerStructSubstate::Identifier,
            identifier: None,
            fields: Vec::new(),
        }
    }
}
