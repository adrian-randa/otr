use std::any::Any;

use crate::{ExpressionParseEnvironment, FallbackExpressionParseEnvironemnt, PunctuationToken::{DoubleColon, Semicolon}, UsingExpressionParseEnvironment, error::CompilerError, expression_parser::ExpressionParser, lexer::token::{KeywordToken, OperatorToken, ParenthesisType, PunctuationToken, Token}};
use otr_core::{error::Result, expression::{AssociatedProcedureCallExpression, Expression, boolean::BooleanExpression, comparison::ComparisonExpression, variable::{VariableAccessMode, VariableAddress, VariableAddressant, VariableExpression}}, procedure::{CompiledProcedure, Instruction}, r#type::Type, value::Value};

trait ScopeExcapeHandler: std::fmt::Debug {
    fn resolve(&self, builder: &mut CompiledProcedureBuilder);

    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
struct IfScopeEscapeHandler {
    target_instruction: usize,
}

impl ScopeExcapeHandler for IfScopeEscapeHandler {
    fn resolve(&self, builder: &mut CompiledProcedureBuilder) {
        builder.instructions.push(Instruction::ShrinkStack);
        builder.shrink_variable_stack();

        let next_ic = builder.instructions.len();

        if let Some(Instruction::JumpConditional {
            condition_expression: _,
            jump_target,
        }) = builder.instructions.get_mut(self.target_instruction)
        {
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
    fn resolve(&self, builder: &mut CompiledProcedureBuilder) {
        builder.instructions.push(Instruction::ShrinkStack);
        builder.shrink_variable_stack();
        builder.instructions.push(Instruction::JumpConditional {
            condition_expression: Expression::Value(Value::Bool(true)),
            jump_target: self.target_instruction,
        });
        let next_ic = builder.instructions.len();
        if let Some(Instruction::JumpConditional {
            condition_expression: _,
            jump_target,
        }) = builder.instructions.get_mut(self.target_instruction)
        {
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
struct ForScopeEscapeHandler {
    start_of_body: usize,
    escape_jump: usize,
}

impl ScopeExcapeHandler for ForScopeEscapeHandler {
    fn resolve(&self, builder: &mut CompiledProcedureBuilder) {
        builder.instructions.push(Instruction::ShrinkStack);
        builder.shrink_variable_stack();
        builder.instructions.push(Instruction::JumpConditional {
            condition_expression: Expression::Value(Value::Bool(true)),
            jump_target: self.start_of_body,
        });

        let end_of_body = builder.instructions.len();

        builder.instructions.push(Instruction::ShrinkStack);
        builder.instructions.push(Instruction::PopVarFromScope {
            identifier: "$CF_FOR_ITER".into(),
        });

        if let Some(Instruction::JumpConditional {
            condition_expression: _,
            jump_target,
        }) = builder.instructions.get_mut(self.escape_jump)
        {
            *jump_target = end_of_body;
        } else {
            panic!("Escape jump not found!");
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
enum CompiledProcedureBuilderState {
    Base,
    Using {
        module_name: Option<String>,
        member_name: Option<Option<String>>,
    },
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
    ForStatement {
        variable_ident: Option<String>,
        in_keyword_read: bool,
        iterator_expression: Vec<Token>,
        parenthesis_index: usize,
    },
    Indeterminate {
        tokens: Vec<Token>,
    },
    Return {
        expression: Vec<Token>,
    },
    Throw {
        expression: Vec<Token>,
    },
}

#[derive(Debug)]
pub struct CompiledProcedureBuilder {
    argument_identifiers: Vec<String>,
    instructions: Vec<Instruction>,
    state: CompiledProcedureBuilderState,
    scope_stack: Vec<Box<dyn ScopeExcapeHandler + 'static>>,
    last_popped_scope: Option<Box<dyn ScopeExcapeHandler + 'static>>,

    variable_stack: Vec<Vec<String>>,
    num_variables: usize,
    stack_top: usize,

    using_expression_parse_environment: UsingExpressionParseEnvironment,
}

impl ExpressionParseEnvironment for CompiledProcedureBuilder {
    fn resolve_procedure_identifier(&self, ident: &dyn AsRef<str>) -> Result<otr_core::module::ModuleAddress> {
        self.using_expression_parse_environment.resolve_procedure_identifier(ident)
    }

    fn resolve_struct_identifier(&self, ident: &dyn AsRef<str>) -> Result<otr_core::module::ModuleAddress> {
        self.using_expression_parse_environment.resolve_struct_identifier(ident)
    }

    fn resolve_variable_ident(&self, ident: &dyn AsRef<str>) -> Result<usize> {
        self.try_resolve_variable_identifier(ident.as_ref())
            .ok_or_else(|| CompilerError::NoSuchVariable { ident: ident.as_ref().to_string() }.boxed())
    }
}

impl CompiledProcedureBuilder {
    pub fn new() -> Self {
        Self {
            argument_identifiers: Vec::new(),
            instructions: Vec::new(),
            state: CompiledProcedureBuilderState::Base,
            scope_stack: Vec::new(),
            last_popped_scope: None,

            variable_stack: vec![Vec::new()],
            num_variables: 0,
            stack_top: 0,

            using_expression_parse_environment: UsingExpressionParseEnvironment::new(),
        }
    }

    pub fn is_scanning(&self) -> bool {
        if let CompiledProcedureBuilderState::Base = self.state {
            false
        } else {
            true
        }
    }

    pub fn push_argument_identifier(mut self, ident: String) -> Result<Self> {
        self.argument_identifiers.push(ident.clone());
        self.declare_variable(ident)?;
        Ok(self)
    }

    pub fn scope_stack_size(&self) -> usize {
        self.scope_stack.len()
    }

    fn try_resolve_variable_identifier(&self, ident: &str) -> Option<usize> {
        let mut i = self.stack_top;

        for stack_idx in (0..self.variable_stack.len()).rev() {
            for var_idx in (0..self.variable_stack[stack_idx].len()).rev() {
                i -= 1;

                if self.variable_stack[stack_idx][var_idx] == ident {
                    return Some(i);
                }
            }
        } 

        None
    }

    fn grow_variable_stack(&mut self) {
        self.variable_stack.push(Vec::new());
    }

    fn shrink_variable_stack(&mut self) {
        let top = self.variable_stack.pop();
        if let Some(top) = top {
            self.stack_top -= top.len();
        }
    }

    pub fn declare_variable(&mut self, ident: String) -> Result<usize> {
        let variables = self.variable_stack.last_mut().unwrap();

        if variables.contains(&ident) {
            Err(CompilerError::VarAlreadyDefined { ident }.boxed())
        } else {
            variables.push(ident.clone());
            self.stack_top += 1;
            self.num_variables = self.num_variables.max(self.stack_top);
            
            Ok(self.stack_top - 1)
        }
    }

    fn parse_variable_address(&self, address: Vec<Token>, environment: &dyn ExpressionParseEnvironment) -> Result<VariableAddress> {
        let mut tokens = address.into_iter();

        let mut addressants = Vec::new();

        while let Some(token) = tokens.next() {
            match token {
                Token::Identifier(ident) => {
                    addressants.push(VariableAddressant::Identifier(ident));
                }
                Token::Punctuation(PunctuationToken::Dot) => {}
                Token::Punctuation(PunctuationToken::SquareBrackets(ParenthesisType::Opening)) => {
                    let index_expression = ExpressionParser::take_until_closing(
                        &mut tokens,
                        Token::Punctuation(PunctuationToken::SquareBrackets(ParenthesisType::Closing)),
                    )?;

                    let index_expression =
                        ExpressionParser::parse(index_expression, environment)?;

                    addressants.push(VariableAddressant::DynamicIndex(index_expression));
                }

                other => {
                    return Err(CompilerError::InvalidScopeAddress {
                        unexpected_token: Some(other),
                    }
                    .boxed());
                }
            }
        }

        if let Some(addressant) =  addressants.get_mut(0)
            && let VariableAddressant::Identifier(ident) = addressant {
                let index = self.try_resolve_variable_identifier(ident)
                    .ok_or_else(|| CompilerError::NoSuchVariable { ident: ident.clone() }.boxed())?;

                *addressant = VariableAddressant::StackIndex(index);
            }

        addressants.try_into().map_err(|_| {
            CompilerError::InvalidScopeAddress {
                unexpected_token: None,
            }
            .boxed()
        })
    }

    pub fn read(
        mut self,
        token: Token,
        expression_parse_environment: &dyn ExpressionParseEnvironment,
    ) -> Result<Self> {
        if let Token::Punctuation(PunctuationToken::Semicolon) = token {
            return self.finish_current_instruction(expression_parse_environment);
        }

        use CompiledProcedureBuilderState::*;
        match &mut self.state {
            Base => match token {
                Token::Keyword(KeywordToken::Let) => {
                    self.state = VarDeclaration {
                        ident: None,
                        expression: None,
                    }
                }
                Token::Keyword(KeywordToken::Using) => {
                    self.state = Using {
                        member_name: None,
                        module_name: None,
                    }
                }
                Token::Keyword(KeywordToken::If) => {
                    self.state = IfStatement {
                        condition_expression: Vec::new(),
                        parenthesis_index: 0,
                    }
                }
                Token::Keyword(KeywordToken::Else) => {
                    let last_scope = self.last_popped_scope.as_ref().ok_or(
                        CompilerError::Unknown {
                            message: "Missing if-clause!".into(),
                        }
                        .boxed(),
                    )?;

                    let if_clause = last_scope
                        .as_any()
                        .downcast_ref::<IfScopeEscapeHandler>()
                        .ok_or(
                            CompilerError::Unknown {
                                message: "else-clauses can only extend 'if' clauses!".into(),
                            }
                            .boxed(),
                        )?;

                    self.state = ElseStatement {
                        original_jump: if_clause.target_instruction,
                    };
                }
                Token::Keyword(KeywordToken::While) => {
                    self.state = WhileStatement {
                        condition_expression: Vec::new(),
                        parenthesis_index: 0,
                    };
                }
                Token::Keyword(KeywordToken::For) => {
                    self.state = ForStatement {
                        variable_ident: None,
                        in_keyword_read: false,
                        iterator_expression: Vec::new(),
                        parenthesis_index: 0,
                    };
                }
                Token::Keyword(KeywordToken::Return) => {
                    self.state = Return {
                        expression: Vec::new(),
                    };
                }
                Token::Keyword(KeywordToken::Throw) => {
                    self.state = Throw {
                        expression: Vec::new(),
                    };
                }

                Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Closing)) => {
                    let handler = self
                        .scope_stack
                        .pop()
                        .ok_or(CompilerError::NoScopeToClose.boxed())?;

                    handler.resolve(&mut self);

                    self.last_popped_scope = Some(handler);
                }

                other => {
                    self.state = Indeterminate {
                        tokens: vec![other],
                    }
                }
            },
            Using { module_name, member_name } => {
                match (token, module_name.as_mut(), member_name.as_mut()) {
                    (Token::Identifier(ident), None, None) => {
                        self.state = Using { module_name: Some(ident), member_name: None };
                    }
                    (Token::Punctuation(DoubleColon), Some(_), None) => {
                        self.state = Using { module_name: module_name.take(), member_name: Some(None) };
                    }
                    (Token::Identifier(ident), Some(_), Some(None)) => {
                        self.state = Using { module_name: module_name.take(), member_name: Some(Some(ident)) };
                    }
                    (Token::Punctuation(Semicolon), _, _) => {
                        self = self.finish_current_instruction(expression_parse_environment)?;
                    }
                    (other, None, None) => {
                        return Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found: other }.boxed());
                    }
                    (other, Some(_), None) => {
                        return Err(CompilerError::UnexpectedToken { expected: Some("::".into()), found: other }.boxed());
                    }
                    (other, Some(_), Some(_)) => {
                        return Err(CompilerError::UnexpectedToken { expected: Some(";".into()), found: other }.boxed());
                    }
                    _ => todo!(),
                }
            }
            VarDeclaration { ident, expression } => {
                if ident.is_none() {
                    if let Token::Identifier(ident) = token {
                        self.state = VarDeclaration {
                            ident: Some(ident),
                            expression: expression.take(),
                        }
                    } else {
                        return Err(CompilerError::UnexpectedToken {
                            expected: Some("Identifier".into()),
                            found: token,
                        }
                        .boxed());
                    }
                } else {
                    if let Some(expr) = expression {
                        expr.push(token);
                    } else {
                        if let Token::Operator(OperatorToken::Assignment) = token {
                            self.state = VarDeclaration {
                                ident: ident.take(),
                                expression: Some(Vec::new()),
                            }
                        } else {
                            return Err(CompilerError::UnexpectedToken {
                                expected: Some("=".into()),
                                found: token,
                            }
                            .boxed());
                        }
                    }
                }
            }
            Assignment {
                address: _,
                expression,
            } => {
                expression.push(token);
            }
            IfStatement {
                condition_expression,
                parenthesis_index,
            } => {
                if let Token::Punctuation(PunctuationToken::Parenthesis(par)) = &token {
                    match par {
                        ParenthesisType::Opening => *parenthesis_index += 1,
                        ParenthesisType::Closing => {
                            if *parenthesis_index > 0 {
                                *parenthesis_index -= 1
                            } else {
                                return Err(CompilerError::InvalidParenthesisStructure.boxed());
                            }
                        }
                    }
                }

                if let Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) =
                    token
                    && *parenthesis_index == 0 {
                        return self.finish_current_instruction(expression_parse_environment);
                    }

                condition_expression.push(token);
            }
            ElseStatement { original_jump: _ } => match token {
                Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) => {
                    return self.finish_current_instruction(expression_parse_environment);
                }

                other => {
                    return Err(CompilerError::UnexpectedToken {
                        expected: Some("{".into()),
                        found: other,
                    }
                    .boxed());
                }
            },
            WhileStatement {
                condition_expression,
                parenthesis_index,
            } => {
                if let Token::Punctuation(PunctuationToken::Parenthesis(par)) = &token {
                    match par {
                        ParenthesisType::Opening => *parenthesis_index += 1,
                        ParenthesisType::Closing => {
                            if *parenthesis_index > 0 {
                                *parenthesis_index -= 1
                            } else {
                                return Err(CompilerError::InvalidParenthesisStructure.boxed());
                            }
                        }
                    }
                }

                if let Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) =
                    token
                    && *parenthesis_index == 0 {
                        return self.finish_current_instruction(expression_parse_environment);
                    }

                condition_expression.push(token);
            }
            ForStatement {
                variable_ident,
                in_keyword_read,
                iterator_expression,
                parenthesis_index,
            } => {
                if variable_ident.is_none() {
                    if let Token::Identifier(ident) = token {
                        *variable_ident = Some(ident);
                        return Ok(self);
                    } else {
                        return Err(CompilerError::UnexpectedToken {
                            expected: Some("Identifier".into()),
                            found: token,
                        }
                        .boxed());
                    }
                }

                if *in_keyword_read {
                    if let Token::Punctuation(PunctuationToken::Parenthesis(par)) = &token {
                        match par {
                            ParenthesisType::Opening => *parenthesis_index += 1,
                            ParenthesisType::Closing => {
                                if *parenthesis_index > 0 {
                                    *parenthesis_index -= 1
                                } else {
                                    return Err(CompilerError::InvalidParenthesisStructure.boxed());
                                }
                            }
                        }
                    }

                    if let Token::Punctuation(PunctuationToken::CurlyBraces(
                        ParenthesisType::Opening,
                    )) = token
                        && *parenthesis_index == 0 {
                            return self.finish_current_instruction(expression_parse_environment);
                        }

                    iterator_expression.push(token);
                } else {
                    if let Token::Keyword(KeywordToken::In) = token {
                        *in_keyword_read = true;
                        return Ok(self);
                    } else {
                        return Err(CompilerError::UnexpectedToken {
                            expected: Some("in".into()),
                            found: token,
                        }
                        .boxed());
                    }
                }
            }
            Indeterminate { tokens } => match token {
                Token::Operator(OperatorToken::Assignment) => {
                    self.state = Assignment {
                        address: tokens.to_vec(),
                        expression: Vec::new(),
                    }
                }

                other => {
                    tokens.push(other);
                }
            },
            Return { expression } => {
                expression.push(token);
            }
            Throw { expression } => {
                expression.push(token);
            }
        }

        Ok(self)
    }

    fn finish_current_instruction(
        mut self,
        expression_parse_environment: &dyn ExpressionParseEnvironment,
    ) -> Result<Self> {
        match self.state {
            CompiledProcedureBuilderState::Base => {},
            CompiledProcedureBuilderState::Using { module_name, member_name } => {
                let module_name = module_name.ok_or_else(|| CompilerError::InvalidDefinition {
                    message: "No module name supplied for 'using' statement".into()
                }.boxed())?;

                let member_name = match member_name {
                    None => None,
                    Some(member_name) => member_name
                };

                self.using_expression_parse_environment.push(module_name, member_name);
                
                self.state = CompiledProcedureBuilderState::Base;
            }
            CompiledProcedureBuilderState::VarDeclaration { ref ident, ref expression } => {
                let ident = ident.clone().ok_or(
                    CompilerError::Unknown {
                        message: "Missing variable identifier!".into(),
                    }
                    .boxed(),
                )?;

                let expression_parse_environment = FallbackExpressionParseEnvironemnt::new(
                    expression_parse_environment,
                    &self
                );

                let expression = expression.as_ref().map(
                    |expression| {
                        ExpressionParser::parse(
                            expression.to_owned(),
                            &expression_parse_environment,
                        )        
                    }
                );

                let stack_index = self.declare_variable(ident)?;

                if let Some(expression) = expression {
                    let expression = expression?;                    

                    self.instructions.push(Instruction::EvaluateExpression {
                        expression,
                        target: Some(
                            vec![VariableAddressant::StackIndex(stack_index)]
                                .try_into()
                                .unwrap(),
                        ),
                    })
                }
            }
            CompiledProcedureBuilderState::Assignment {
                ref address,
                ref expression,
            } => {
                let expression_parse_environment = FallbackExpressionParseEnvironemnt::new(
                    expression_parse_environment,
                    &self
                );

                let target = Some(self.parse_variable_address(address.to_owned(), &expression_parse_environment)?);


                let expression = ExpressionParser::parse(expression.to_owned(), &expression_parse_environment)?;

                self.instructions.push(Instruction::EvaluateExpression { expression, target });
            }
            CompiledProcedureBuilderState::IfStatement {
                ref condition_expression,
                parenthesis_index,
            } => {
                if parenthesis_index > 0 {
                    return Err(CompilerError::InvalidParenthesisStructure.boxed());
                }

                let expression_parse_environment = FallbackExpressionParseEnvironemnt::new(
                    expression_parse_environment,
                    &self
                );

                let condition_expression =
                    Expression::Boolean(BooleanExpression::Not(Box::new(ExpressionParser::parse(
                        condition_expression.to_owned(),
                        &expression_parse_environment,
                    )?)));

                self.scope_stack.push(Box::new(IfScopeEscapeHandler {
                    target_instruction: self.instructions.len(),
                }));

                self.instructions.push(Instruction::JumpConditional {
                    condition_expression,
                    jump_target: usize::MAX,
                });
                self.instructions.push(Instruction::GrowStack);
                self.grow_variable_stack();
            }
            CompiledProcedureBuilderState::ElseStatement { original_jump } => {
                let instruction = &mut self.instructions[original_jump];

                match instruction {
                    Instruction::JumpConditional { condition_expression: _, jump_target } => {
                        *jump_target += 1;

                        self.scope_stack.push(
                            Box::new(IfScopeEscapeHandler { target_instruction: self.instructions.len() })
                        );

                        self.instructions.push(Instruction::JumpConditional {
                            condition_expression: Expression::Value(Value::Bool(true)),
                            jump_target: usize::MAX
                        });

                        self.instructions.push(
                            Instruction::GrowStack
                        );
                        self.grow_variable_stack();
                    }

                    _ => {
                        return Err(CompilerError::Unknown {
                            message: "Instruction referenced by 'if' scope handler is not of type JumpConditional!".into()
                        }.boxed())
                    }
                }
            }
            CompiledProcedureBuilderState::WhileStatement {
                ref condition_expression,
                parenthesis_index,
            } => {
                if parenthesis_index > 0 {
                    return Err(CompilerError::InvalidParenthesisStructure.boxed());
                }

                let expression_parse_environment = FallbackExpressionParseEnvironemnt::new(
                    expression_parse_environment,
                    &self
                );

                let condition_expression =
                    Expression::Boolean(BooleanExpression::Not(Box::new(ExpressionParser::parse(
                        condition_expression.to_owned(),
                        &expression_parse_environment,
                    )?)));

                self.scope_stack.push(Box::new(WhileScopeEscapeHandler {
                    target_instruction: self.instructions.len(),
                }));

                self.instructions.push(Instruction::JumpConditional {
                    condition_expression,
                    jump_target: usize::MAX,
                });
                self.instructions.push(Instruction::GrowStack);
                self.grow_variable_stack();
            }
            CompiledProcedureBuilderState::ForStatement {
                ref variable_ident,
                in_keyword_read: _,
                ref iterator_expression,
                parenthesis_index,
            } => {
                if parenthesis_index > 0 {
                    return Err(CompilerError::InvalidParenthesisStructure.boxed());
                }

                let parsed_iterator_expression;

                {
                    let expression_parse_environment = FallbackExpressionParseEnvironemnt::new(
                        expression_parse_environment,
                        &self
                    );
    
                    parsed_iterator_expression = ExpressionParser::parse(
                        iterator_expression.to_owned(),
                        &expression_parse_environment,
                    )?;
                }

                let iterator_expression =
                    Expression::AssociatedProcedureCall(AssociatedProcedureCallExpression::new(
                        Box::new(parsed_iterator_expression),
                        "intoIterator".into(),
                        Vec::new(),
                    ));

                let variable_ident = variable_ident.clone().ok_or(
                    CompilerError::Unknown {
                        message: "No given variable identifier!".into(),
                    }
                    .boxed(),
                )?;
                
                let iter_idx = self.declare_variable("$CF_FOR_ITER".into()).or_else(|_| self.resolve_variable_ident(&"$CF_FOR_ITER"))?;

                self.instructions.push(Instruction::EvaluateExpression {
                    expression: iterator_expression,
                    target: Some(
                        vec![VariableAddressant::StackIndex(iter_idx)]
                            .try_into()
                            .unwrap(),
                    ),
                });

                // Start of body
                let start_of_body = self.instructions.len();

                // Compute next
                self.instructions.push(Instruction::GrowStack);
                self.grow_variable_stack();
                self.instructions.push(Instruction::PushVarToScope {
                    identifier: variable_ident.clone(),
                });
                let var_idx = self.declare_variable(variable_ident.clone())?;
                self.instructions.push(Instruction::EvaluateExpression {
                    expression: Expression::AssociatedProcedureCall(
                        AssociatedProcedureCallExpression::new(
                            Box::new(Expression::Variable(VariableExpression::new(
                                vec![VariableAddressant::StackIndex(iter_idx)]
                                    .try_into()
                                    .unwrap(),
                                VariableAccessMode::Ref,
                            ))),
                            "next".into(),
                            Vec::new(),
                        ),
                    ),
                    target: Some(
                        vec![VariableAddressant::StackIndex(var_idx)]
                            .try_into()
                            .unwrap(),
                    ),
                });

                // Scope escape if next is null
                let escape_jump = self.instructions.len();
                self.instructions.push(Instruction::JumpConditional {
                    condition_expression: Expression::Comparison(ComparisonExpression::Equals {
                        lhs: Box::new(Expression::Value(Value::Type(Type::Null))),
                        rhs: Box::new(Expression::Variable(VariableExpression::new(
                            vec![VariableAddressant::StackIndex(var_idx)]
                                .try_into()
                                .unwrap(),
                            VariableAccessMode::TypeOf,
                        ))),
                    }),
                    jump_target: usize::MAX,
                });

                self.scope_stack.push(Box::new(ForScopeEscapeHandler {
                    start_of_body,
                    escape_jump,
                }));
            }
            CompiledProcedureBuilderState::Indeterminate { ref tokens } => {
                let expression_parse_environment = FallbackExpressionParseEnvironemnt::new(
                    expression_parse_environment,
                    &self
                );

                let expression =
                    ExpressionParser::parse(tokens.to_owned(), &expression_parse_environment)?;

                self.instructions.push(Instruction::EvaluateExpression {
                    expression,
                    target: None,
                });
            }
            CompiledProcedureBuilderState::Return { ref expression } => {
                let expression_parse_environment = FallbackExpressionParseEnvironemnt::new(
                    expression_parse_environment,
                    &self
                );

                let expression = if expression.is_empty() {
                    Expression::Value(Value::Null)
                } else {
                    ExpressionParser::parse(expression.to_owned(), &expression_parse_environment)?
                };

                self.instructions.push(Instruction::Return { expression });
            }
            CompiledProcedureBuilderState::Throw { ref expression } => {
                let expression_parse_environment = FallbackExpressionParseEnvironemnt::new(
                    expression_parse_environment,
                    &self
                );

                let expression = if expression.is_empty() {
                    Expression::Value(Value::Null)
                } else {
                    ExpressionParser::parse(expression.to_owned(), &expression_parse_environment)?
                };

                self.instructions.push(Instruction::Throw { expression });
            }
        }
        self.state = CompiledProcedureBuilderState::Base;
        Ok(self)
    }

    pub fn build(self) -> Result<CompiledProcedure> {
        if let CompiledProcedureBuilderState::Base = self.state {
            if !self.scope_stack.is_empty() {
                return Err(CompilerError::Unknown {
                    message: "Unclosed scope!".into(),
                }
                .boxed());
            }
            

            Ok(CompiledProcedure {
                instructions: self.instructions,
                num_args: self.argument_identifiers.len(),
                stack_size: self.num_variables,
            })
        } else {
            Err(CompilerError::Unknown {
                message: "Incomplete instruction!".into(),
            }
            .boxed())
        }
    }
}
