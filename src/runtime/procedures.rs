use std::{any::Any, collections::HashMap, rc::Rc};

use crate::{compiler::{CompilerError, expression_parser::ExpressionParser}, lexer::token::{KeywordToken, OperatorToken, ParenthesisType, PunctuationToken, Token}, runtime::{
    Environment, Expression, RuntimeError, Scope, ScopeAddress, ScopeAddressant, Value, expressions::boolean::NotExpression,
}};

pub trait Procedure: std::fmt::Debug {
    fn call(&self, environment: Environment, arguments: Vec<Value>) -> Result<Value, RuntimeError>;
}

#[derive(Debug)]
pub enum Instruction {
    //TODO: Remove public viisibility
    PushVarToScope {
        identifier: String,
    },
    PopVarFromScope {
        identifier: String,
    },
    GrowStack,
    ShrinkStack,
    EvaluateExpression {
        expression: Box<dyn Expression>,
        target: Option<ScopeAddress>,
    },
    JumpConditional {
        condition_expression: Box<dyn Expression>,
        jump_target: usize,
    },
    Return {
        expression: Box<dyn Expression>,
    },
}

#[derive(Debug)]
pub struct CompiledProcedure {
    //TODO: Remove public visibility
    pub arguments_identifiers: Vec<String>,
    pub instructions: Vec<Instruction>,
}

impl Procedure for CompiledProcedure {
    fn call(
        &self,
        mut environment: Environment,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let members = HashMap::from_iter(
            self.arguments_identifiers
                .clone()
                .into_iter()
                .zip(arguments.into_iter()),
        );

        environment.insert_members(members);

        let mut pc = 0;

        while pc < self.instructions.len() {
            match &self.instructions[pc] {
                Instruction::PushVarToScope { identifier } => {
                    environment.scope.push(identifier.clone())?;
                }
                Instruction::PopVarFromScope { identifier } => {
                    environment.scope.pop(identifier)?;
                }
                Instruction::GrowStack => {
                    environment.scope.grow_stack();
                }
                Instruction::ShrinkStack => {
                    environment.scope.shrink_stack();
                }
                Instruction::EvaluateExpression { expression, target } => {
                    let eval_result = expression.eval(&mut environment)?;

                    if let Some(target) = target {
                        environment.set_variable(target.clone(), eval_result)?;
                    }
                }
                Instruction::JumpConditional {
                    condition_expression: procedure,
                    jump_target,
                } => {
                    let returned_value = procedure.eval(&mut environment)?;

                    match returned_value {
                        Value::Bool(value) => {
                            if value {
                                pc = *jump_target;
                                continue;
                            }
                        }
                        _ => {
                            return Err(RuntimeError {
                                message: format!(
                                    "Expected Bool, found {}!",
                                    returned_value.get_type_id()
                                ),
                            })
                        }
                    }
                }
                Instruction::Return {
                    expression: procedure,
                } => return procedure.eval(&mut environment),
            }

            pc += 1;
        }

        Ok(Value::Null)
    }
}



trait ScopeExcapeHandler: std::fmt::Debug {
    fn resolve(&self, instructions: &mut Vec<Instruction>);

    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
struct IfScopeEscapeHandler {
    target_instruction: usize,
}

impl ScopeExcapeHandler for IfScopeEscapeHandler {
    fn resolve(&self, instructions: &mut Vec<Instruction>) {
        instructions.push(Instruction::ShrinkStack);

        let next_ic = instructions.len();

        if let Some(Instruction::JumpConditional {
            condition_expression: _,
            jump_target 
        }) = instructions.get_mut(self.target_instruction) {
            *jump_target = next_ic;
        } else {
            panic!("Tried resolving if scope escape but initial jump is missing!");
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
struct WhileScopeEscapeHandler {
    target_instruction: usize,
}

impl ScopeExcapeHandler for WhileScopeEscapeHandler {
    fn resolve(&self, instructions: &mut Vec<Instruction>) {
        instructions.push(Instruction::ShrinkStack);
        instructions.push(Instruction::JumpConditional {
            condition_expression: Box::new(Value::Bool(true)),
            jump_target: self.target_instruction
        });
        let next_ic = instructions.len();
        if let Some(Instruction::JumpConditional {
            condition_expression: _,
            jump_target 
        }) = instructions.get_mut(self.target_instruction) {
            
            *jump_target = next_ic;
        } else {
            panic!("Tried resolving if scope escape but initial jump is missing!");
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
enum CompiledProcedureBuilderState {
    Base,
    VarDeclaration {
        ident: Option<String>,
        expression: Option<Vec<Token>>,
    },
    Assignment {
        address: Vec<Token>,
        expression: Vec<Token>,
    },
    IfStatement {
        condition_expression: Vec<Token>,
        parenthesis_index: usize,
    },
    ElseStatement {
        original_jump: usize,
    },
    WhileStatement {
        condition_expression: Vec<Token>,
        parenthesis_index: usize,
    },
    Indeterminate {
        tokens: Vec<Token>,
    },
    Return {
        expression: Vec<Token>,
    }
}

#[derive(Debug)]
pub struct CompiledProcedureBuilder {
    procedure: CompiledProcedure,
    state: CompiledProcedureBuilderState,
    scope_stack: Vec<Box<dyn ScopeExcapeHandler + 'static>>,
    last_popped_scope: Option<Box<dyn ScopeExcapeHandler + 'static>>,
}

impl CompiledProcedureBuilder {
    pub fn new() -> Self {
        Self {
            procedure: CompiledProcedure { arguments_identifiers: Vec::new(), instructions: Vec::new() },
            state: CompiledProcedureBuilderState::Base,
            scope_stack: Vec::new(),
            last_popped_scope: None,
        }
    }

    pub fn is_scanning(&self) -> bool {
        if let CompiledProcedureBuilderState::Base = self.state {
            false
        } else {
            true
        }
    }

    pub fn push_argument_identifier(mut self, ident: String) -> Self {
        self.procedure.arguments_identifiers.push(ident);
        self
    }

    pub fn scope_stack_size(&self) -> usize {
        self.scope_stack.len()
    }

    pub fn read(mut self, token: Token) -> Result<Self, CompilerError> {

        if let Token::Punctuation(PunctuationToken::Semicolon) = token {
            return self.finish_current_instruction()
        }

        use CompiledProcedureBuilderState::*;
        match &mut self.state {
            Base => {
                match token {
                    Token::Keyword(KeywordToken::Let) => {
                        self.state = VarDeclaration { ident: None, expression: None }
                    }
                    Token::Keyword(KeywordToken::If) => {
                        self.state = IfStatement { condition_expression: Vec::new(), parenthesis_index: 0 }
                    }
                    Token::Keyword(KeywordToken::Else) => {
                        let last_scope = self.last_popped_scope.as_ref()
                            .ok_or(CompilerError {
                                message: "Missing if-clause!".into()
                            })?;
                        
                        let if_clause = last_scope.as_any()
                            .downcast_ref::<IfScopeEscapeHandler>().ok_or(CompilerError {
                                message: "else-clauses can only extend 'if' clauses!".into()
                            })?;
                        
                        self.state = ElseStatement { original_jump: if_clause.target_instruction };
                    }
                    Token::Keyword(KeywordToken::While) => {
                        self.state = WhileStatement { condition_expression: Vec::new(), parenthesis_index: 0 }
                    }
                    Token::Keyword(KeywordToken::Return) => {
                        self.state = Return { expression: Vec::new() }
                    }

                    Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Closing)) => {
                        let handler = self.scope_stack
                            .pop()
                            .ok_or(CompilerError {
                                message: "Invalid closing curly brace!".into()
                            })?;
                        
                        handler.resolve(&mut self.procedure.instructions);

                        
                        
                        self.last_popped_scope = Some(handler);
                    }

                    other => {
                        self.state = Indeterminate { tokens: vec![other] }
                    }
                }
            },
            VarDeclaration { ident, expression } => {
                if ident.is_none() {
                    if let Token::Identifier(ident) = token {
                        self.state = VarDeclaration { ident: Some(ident), expression: expression.take() }
                    } else {
                        return Err(CompilerError {
                            message: format!("Unexprected token. Expected identifier, found {:?}!", token)
                        });
                    }
                } else {
                    if let Some(expr) = expression {
                        expr.push(token);
                    } else {
                        if let Token::Operator(OperatorToken::Assignment) = token {
                            self.state = VarDeclaration { ident: ident.take(), expression: Some(Vec::new()) }
                        } else {
                            return Err(CompilerError {
                                message: format!("Unexprected token. Expected '=', found {:?}!", token)
                            });
                        }
                    }
                }
            },
            Assignment { address, expression } => {
                expression.push(token);
            },
            IfStatement { condition_expression, parenthesis_index } => {
                if let Token::Punctuation(PunctuationToken::Parenthesis(par)) = &token {
                    match par {
                        ParenthesisType::Opening => *parenthesis_index += 1,
                        ParenthesisType::Closing => if *parenthesis_index > 0 {
                            *parenthesis_index -= 1
                        } else {
                            return Err(CompilerError { message: "Invalid parenthesis structure!".into() })
                        },
                    }
                }

                if let Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) = token {
                    if *parenthesis_index == 0 {
                        return self.finish_current_instruction()
                    }
                }

                condition_expression.push(token);
            },
            ElseStatement { original_jump: _ } => {
                match token {
                    Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) => {
                        return self.finish_current_instruction();
                    }

                    other => {
                        return Err(CompilerError {
                            message: format!("Unexpected token. Expected '{{', found {:?}!", other)
                        });
                    }
                }
            }
            WhileStatement { condition_expression, parenthesis_index } => {
                if let Token::Punctuation(PunctuationToken::Parenthesis(par)) = &token {
                    match par {
                        ParenthesisType::Opening => *parenthesis_index += 1,
                        ParenthesisType::Closing => if *parenthesis_index > 0 {
                            *parenthesis_index -= 1
                        } else {
                            return Err(CompilerError { message: "Invalid parenthesis structure!".into() })
                        },
                    }
                }

                if let Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) = token {
                    if *parenthesis_index == 0 {
                        return self.finish_current_instruction()
                    }
                }

                condition_expression.push(token);
            },
            Indeterminate { tokens } => {
                match token {
                    Token::Operator(OperatorToken::Assignment) => {
                        self.state = Assignment { address: tokens.to_vec(), expression: Vec::new() }
                    }

                    other => {
                        tokens.push(other);
                    }
                }
            },
            Return { expression } => {
                expression.push(token);
            },
        }


        Ok(self)
    }

    fn finish_current_instruction(mut self) -> Result<Self, CompilerError> {
        match &mut self.state {
            CompiledProcedureBuilderState::Base => {
            },
            CompiledProcedureBuilderState::VarDeclaration { ident, expression } => {
                let ident = ident.clone().ok_or(CompilerError {
                    message: "Missing variable identifier!".into()
                })?;
                self.procedure.instructions.push(
                    Instruction::PushVarToScope { identifier: ident.clone() }
                );
                if let Some(expression) = expression {
                    let expression = ExpressionParser::parse(expression.to_owned())?;

                    self.procedure.instructions.push(
                        Instruction::EvaluateExpression { expression, target: Some(vec![
                            ScopeAddressant::Identifier(ident)
                        ].try_into().unwrap()) }
                    )
                }
            },
            CompiledProcedureBuilderState::Assignment { address, expression } => {
                let target = Some(ScopeAddress::try_from(address.to_owned())?);

                let expression = ExpressionParser::parse(expression.to_owned())?;

                self.procedure.instructions.push(Instruction::EvaluateExpression { expression, target });
            },
            CompiledProcedureBuilderState::IfStatement { condition_expression, parenthesis_index } => {
                if *parenthesis_index > 0 {
                    return Err(CompilerError {
                        message: "Invalid parenthesis structure!".into()
                     });
                }

                let condition_expression = Box::new(NotExpression::new(
                    ExpressionParser::parse(condition_expression.to_owned())?
                ));

                self.scope_stack.push(
                    Box::new(IfScopeEscapeHandler { target_instruction: self.procedure.instructions.len() })
                );
                
                self.procedure.instructions.push(
                    Instruction::JumpConditional { condition_expression, jump_target: usize::MAX }
                );
                self.procedure.instructions.push(
                    Instruction::GrowStack
                );
            },
            CompiledProcedureBuilderState::ElseStatement { original_jump } => {
                let instruction = &mut self.procedure.instructions[*original_jump];

                match instruction {
                    Instruction::JumpConditional { condition_expression: _, jump_target } => {
                        *jump_target += 1;

                        self.scope_stack.push(
                            Box::new(IfScopeEscapeHandler { target_instruction: self.procedure.instructions.len() })
                        );

                        self.procedure.instructions.push(Instruction::JumpConditional {
                            condition_expression: Box::new(Value::Bool(true)),
                            jump_target: usize::MAX
                        });

                        self.procedure.instructions.push(
                            Instruction::GrowStack
                        );
                    }

                    _ => {
                        return Err(CompilerError {
                            message: "Instruction referenced by 'if' scope handler is not of type JumpConditional!".into()
                        })
                    }
                }
            }
            CompiledProcedureBuilderState::WhileStatement { condition_expression, parenthesis_index } => {
                if *parenthesis_index > 0 {
                    return Err(CompilerError {
                        message: "Invalid parenthesis structure!".into()
                     });
                }

                let condition_expression = Box::new(NotExpression::new(
                    ExpressionParser::parse(condition_expression.to_owned())?
                ));

                
                self.scope_stack.push(
                    Box::new(WhileScopeEscapeHandler { target_instruction: self.procedure.instructions.len() })
                );
                
                self.procedure.instructions.push(
                    Instruction::JumpConditional { condition_expression, jump_target: usize::MAX }
                );
                self.procedure.instructions.push(Instruction::GrowStack);
            },
            CompiledProcedureBuilderState::Indeterminate { tokens } => {
                let expression = ExpressionParser::parse(tokens.to_owned())?;

                self.procedure.instructions.push(
                    Instruction::EvaluateExpression { expression, target: None }
                );
            },
            CompiledProcedureBuilderState::Return { expression } => {
                let expression = if expression.is_empty() {
                    Box::new(Value::Null)
                } else {
                    ExpressionParser::parse(expression.to_owned())?
                };

                self.procedure.instructions.push(
                    Instruction::Return { expression }
                );
            },
        }
        self.state = CompiledProcedureBuilderState::Base;
        Ok(self)
    }

    pub fn build(self) -> Result<CompiledProcedure, CompilerError> {
        if let CompiledProcedureBuilderState::Base = self.state {
            if !self.scope_stack.is_empty() {
                return Err(CompilerError {
                    message: "Unclosed scope!".into()
                });
            }

            Ok(self.procedure)
        } else {
            Err(CompilerError {
                message: "Incomplete instruction!".into()
            })
        }
    }
}


pub mod builtin;
