use otr_core::expression::Operator;

use crate::{CompilerState, Module, ParenthesisType, PunctuationToken, Token, error::CompilerError, parenthesis::ParenthesisStack, procedure::CompiledProcedureBuilder, states::module::CompilerModuleState};


pub(crate) enum CompilerOperatorOverloadState {
    Base { module: Box<CompilerModuleState> } ,
    AfterStructIdentifier { module: Box<CompilerModuleState>, struct_identifier: String },
    AfterOperator { module: Box<CompilerModuleState>, struct_identifier: String, operator: Operator },
    InsideScope { module: Box<CompilerModuleState>, struct_identifier: String, operator: Operator, procedure: CompiledProcedureBuilder },
}

impl CompilerOperatorOverloadState {
    pub(crate) fn new(module: Box<CompilerModuleState>) -> Self {
        Self::Base { module }
    }
}

impl CompilerState for CompilerOperatorOverloadState {
    fn read(
        mut self: Box<Self>,
        token: Token,
        compiler_environment: &mut crate::CompilerEnvironment,
    ) -> otr_core::Result<Box<dyn CompilerState>> {
        use CompilerOperatorOverloadState::*;

        match *self {
            Base { module } => {
                if let Token::Identifier(struct_identifier) = token {
                    *self = AfterStructIdentifier { module, struct_identifier };
                    Ok(self)
                } else {
                    Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found: token }.boxed())
                }
            },
            AfterStructIdentifier { module, struct_identifier } => {
                if let Token::Operator(operator) = token {
                    if let Some(operator) = operator.try_into_core_operator() {
                        *self = AfterOperator { module, struct_identifier, operator };
                        Ok(self)
                    } else {
                        Err(CompilerError::InvalidDefinition { message: format!("Cannot overload operator '{:?}'!", operator) }.boxed())
                    }
                } else {
                    Err(CompilerError::UnexpectedToken { expected: Some("Operator".into()), found: token }.boxed())
                }
            },
            AfterOperator { module, struct_identifier, operator } => {
                if matches!(token, Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening))) {
                    let mut parenthesis_stack = ParenthesisStack::new();
                    parenthesis_stack.read(token).unwrap();
                    let mut procedure = CompiledProcedureBuilder::new()
                            .push_argument_identifier("this".into())?;
                    if !matches!(operator, Operator::Not) {
                        procedure = procedure.push_argument_identifier("other".into())?;
                    }

                    *self = InsideScope { module, struct_identifier, operator, procedure };
                    Ok(self)
                } else {
                    Err(CompilerError::UnexpectedToken { expected: Some("'{'".into()), found: token }.boxed())
                }
            },
            InsideScope { mut module, struct_identifier, operator, mut procedure } => {
                if matches!(token, Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Closing)))
                    && procedure.scope_stack_size() == 0 && !procedure.is_scanning()
                {
                    let procedure = procedure.build()?;

                    module.get_module_mut().insert_operator(struct_identifier, operator, Box::new(procedure), false);

                    Ok(module)
                } else {
                    procedure = procedure.read(token, compiler_environment)?;
                    *self = InsideScope { module, struct_identifier, operator, procedure };
                    Ok(self)
                }
            },
        }
    }

    fn finalize(self: Box<Self>) -> otr_core::Result<Module> {
        Err(CompilerError::Unknown { message: "Unfinished operator overload definition!".into() }.boxed())
    }
}