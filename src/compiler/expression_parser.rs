use std::{collections::HashMap, rc::Rc};

use crate::{compiler::CompilerError, lexer::token::{OperatorToken, ParenthesisType, PunctuationToken, Token}, runtime::{Expression, ModuleAddress, ScopeAddress, ScopeAddressant, Value, expressions::{EqualityExpression, ProcedureCallExpression, StructConstructionExpression, VariableExpression, arithmetic::{AddExpression, DivideExpression, GreaterThanExpression, ModuloExpression, MultiplyExpression, PowerExpression, SubtractExpression}, boolean::{AndExpression, NotExpression, OrExpression}}}};

#[derive(Debug)]
pub enum ExpressionAtom {
    Subexpression(Box<dyn Expression>),
    Operator(OperatorToken)
}

impl ExpressionAtom {
    fn unwrap_subexpression(self) -> Box<dyn Expression> {
        match self {
            ExpressionAtom::Subexpression(expression) => expression,
            ExpressionAtom::Operator(_) => panic!("Called unwrap on non subexpression!"),
        }
    }
}

#[derive(Debug)]
pub enum RawExpressionAtom {
    Subexpression(Vec<Token>),
    Operator(OperatorToken),
}

pub struct ExpressionParser;

impl ExpressionParser {
    pub fn parse(expression: impl IntoIterator<Item = Token>) -> Result<Box<dyn Expression>, CompilerError> {
        let atoms = Self::atomize(expression)?;

        let mut operator_order = Vec::new();
        for i in 0..atoms.len() {
            if let ExpressionAtom::Operator(operator) = &atoms[i] {
                operator_order.push((Self::get_precedence(operator), i));
            }
        }
        operator_order.sort_by_key(|(precedence, _i)| usize::MAX - *precedence);

        let mut atoms = atoms
            .into_iter()
            .map(|atom| Some(atom))
            .collect::<Vec<_>>();

        for i in 0..operator_order.len() {
            if let Some(ExpressionAtom::Operator(op)) = atoms[operator_order[i].1].take() {
                match op {
                    OperatorToken::Not => {
                        if let Some(ExpressionAtom::Subexpression(subexpr)) = atoms[operator_order[i].1 + 1].take() {
                            let splice = vec![Some(ExpressionAtom::Subexpression(
                                Box::new(NotExpression::new(subexpr))
                            ))];

                            atoms.splice(i..=i+1, splice);

                            for operator in &mut operator_order {
                                if operator.1 > i {
                                    *operator = (operator.0, operator.1 - 2);
                                }
                            }
                        }
                    }

                    op => {
                        if operator_order[i].1 == 0 {
                            return Err(CompilerError { message: "Expressions may not start with a binary operator!".into() });
                        }
                        if let (
                            Some(ExpressionAtom::Subexpression(lhs)),
                            Some(ExpressionAtom::Subexpression(rhs))
                        ) = (
                            atoms[operator_order[i].1 - 1].take(),
                            atoms[operator_order[i].1 + 1].take()
                        ) {
                            let splice = vec![Some(ExpressionAtom::Subexpression(
                                Self::resolve_binary_operator(&op, lhs, rhs)?
                            ))];
                            let op_index = operator_order[i].1;

                            atoms.splice(
                                (operator_order[i].1 - 1)..=(operator_order[i].1 + 1),
                                splice
                            );

                            for operator in &mut operator_order {
                                if operator.1 > op_index {
                                    *operator = (operator.0, operator.1 - 2);
                                }
                            }
                        }
                    }
                }

            } else {
                Err(CompilerError {
                    message: "Missing operator!".into()
                })?;
            }
        }

        Ok(atoms[0].take().unwrap().unwrap_subexpression())
    }

    pub fn atomize(expression: impl IntoIterator<Item = Token>) -> Result<Vec<ExpressionAtom>, CompilerError> {
        let raw_atoms = Self::split(expression)?;

        let mut atoms = Vec::new();

        for atom in raw_atoms {
           atoms.push(Self::parse_raw_atom(atom)?);
        }

        Ok(atoms)
    }

    pub fn take_until_closing(tokens: impl IntoIterator<Item = Token>, parenthesis: Token) -> Result<Vec<Token>, CompilerError> {
        use PunctuationToken::*;

        let mut stack = Vec::new();

        match parenthesis {
            Token::Punctuation(PunctuationToken::Parenthesis(_)) => {
                stack.push(Parenthesis(ParenthesisType::Opening));
            }
            Token::Punctuation(PunctuationToken::SquareBrackets(_)) => {
                stack.push(SquareBrackets(ParenthesisType::Opening));
            }
            Token::Punctuation(PunctuationToken::CurlyBraces(_)) => {
                stack.push(CurlyBraces(ParenthesisType::Opening));
            }

            _ => panic!("Unsupported parenthesis type!")
        };

        let mut slice = Vec::new();

        let mut iter = tokens.into_iter();

        while let Some(token) = iter.next() {
            if stack.len() == 1 && &token == &parenthesis {
                return Ok(slice);
            }
            match token.clone() {
                Token::Punctuation(punct) => {
                    
                    match &punct {
                        Parenthesis(p) |
                        SquareBrackets(p) |
                        CurlyBraces(p) => {
                            match p {
                                ParenthesisType::Opening => stack.push(punct),
                                ParenthesisType::Closing => {
                                    let top = stack.pop().ok_or(CompilerError {
                                        message: "Invalid parenthesis structure!".into()
                                    })?;

                                    match (&top, &punct) {
                                        (Parenthesis(_), Parenthesis(_)) |
                                        (SquareBrackets(_), SquareBrackets(_)) |
                                        (CurlyBraces(_), CurlyBraces(_)) => {}
                                        _ => {
                                            return Err(CompilerError { message: "Invalid parenthesis structure!".into() });
                                        }                                        
                                    }
                                },
                            }
                        }

                        _ => {}
                    };

                    slice.push(token);
                }

                token => {
                    slice.push(token);
                }
            }
        }

        if !stack.is_empty() {
            return Err(CompilerError {
                message: "Invalid parenthesis structure!".into()
            });
        }

        Ok(slice)
    }


    pub fn split_by_commas(tokens: impl IntoIterator<Item = Token>) -> Result<Vec<Vec<Token>>, CompilerError> {

        let mut iter = tokens.into_iter();

        let mut slices = Vec::new();

        let mut current = Vec::new();

        let mut stack = Vec::new();        

        while let Some(next) = iter.next() {
            if let Token::Punctuation(punct) = next.clone() {
                use PunctuationToken::*;

                match &punct {
                    Parenthesis(p) |
                    SquareBrackets(p) |
                    CurlyBraces(p) => {
                        match p {
                            ParenthesisType::Opening => stack.push(punct),
                            ParenthesisType::Closing => {
                                let top = stack.pop().ok_or(CompilerError {
                                    message: "Invalid parenthesis structure!".into()
                                })?;

                                match (&top, &punct) {
                                    (Parenthesis(_), Parenthesis(_)) |
                                    (SquareBrackets(_), SquareBrackets(_)) |
                                    (CurlyBraces(_), CurlyBraces(_)) => {}
                                    _ => {
                                        return Err(CompilerError { message: "Invalid parenthesis structure!".into() });
                                    }                                        
                                }
                            },
                        }
                    }

                    _ => {}
                };
            }

            if let Token::Punctuation(PunctuationToken::Comma) = next {
                if stack.is_empty() {
                    slices.push(current);
                    current = Vec::new();
                    continue;
                }
            }

            current.push(next);
        }

        if !current.is_empty() {
            slices.push(current);
        }

        Ok(slices)
    }

    pub fn split(tokens: impl IntoIterator<Item = Token>) -> Result<Vec<RawExpressionAtom>, CompilerError> {
        let mut tokens = tokens.into_iter();

        let mut atoms = Vec::new();
        let mut current_subexpression = Vec::new();

        let mut stack = Vec::new();   

        while let Some(next) = tokens.next() {
            match next.clone() {
                Token::Punctuation(punct) => {
                    use PunctuationToken::*;

                    match &punct {
                        Parenthesis(p) |
                        SquareBrackets(p) |
                        CurlyBraces(p) => {
                            match p {
                                ParenthesisType::Opening => stack.push(punct),
                                ParenthesisType::Closing => {
                                    let top = stack.pop().ok_or(CompilerError {
                                        message: "Invalid parenthesis structure!".into()
                                    })?;

                                    match (&top, &punct) {
                                        (Parenthesis(_), Parenthesis(_)) |
                                        (SquareBrackets(_), SquareBrackets(_)) |
                                        (CurlyBraces(_), CurlyBraces(_)) => {}
                                        _ => {
                                            return Err(CompilerError { message: "Invalid parenthesis structure!".into() });
                                        }                                        
                                    }
                                },
                            }
                        }

                        _ => {}
                    };
                }

                Token::Operator(operator) => {
                    if stack.is_empty() {
                        if !current_subexpression.is_empty() {
                            atoms.push(RawExpressionAtom::Subexpression(current_subexpression));
                        }
                        current_subexpression = Vec::new();
                        atoms.push(RawExpressionAtom::Operator(operator));
                        continue;
                    }
                }

                _ => {}
            }
            current_subexpression.push(next);
        }

        atoms.push(RawExpressionAtom::Subexpression(current_subexpression));

        Ok(atoms)
    }

    fn parse_raw_atom(atom: RawExpressionAtom) -> Result<ExpressionAtom, CompilerError> {
        match atom {
            RawExpressionAtom::Operator(operator) => Ok(ExpressionAtom::Operator(operator)),
            RawExpressionAtom::Subexpression(tokens) => {
                // Epmpty
                if tokens.len() == 0 {
                    return Err(CompilerError {
                        message: "Found empty subexpression atom!".into()
                    });
                }

                // Single token
                if tokens.len() == 1 {
                    let token = &tokens[0];
                    match token {
                        Token::Literal(literal) => {
                            return Ok(ExpressionAtom::Subexpression(Box::new(Value::try_from(literal.to_owned())?)))
                        }
                        Token::Identifier(ident) => {
                            return Ok(ExpressionAtom::Subexpression(Box::new(VariableExpression {
                                variable_address: vec![ScopeAddressant::Identifier(ident.to_owned())]
                                    .try_into()
                                    .map_err(|_| CompilerError {
                                        message: format!("Could not resolve identifier '{}'!", ident)
                                    })?
                            })))
                        }
                        _ => {
                            return Err(CompilerError {
                                message: format!("Unexpected token. Expected literal or identifier, found {:?}", token)
                            });
                        }
                    }
                }

                if let Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Opening)) = tokens[0] {
                    let mut tokens = tokens.into_iter().skip(1);
                    let subexpression = Self::take_until_closing(
                        &mut tokens,
                        Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Closing))
                    )?;

                    if let Some(token) = tokens.next() {
                        Err(CompilerError {
                            message: format!("Unexpected token. Expected operator, found {:?}", token)
                        })?;
                    }

                    return Ok(ExpressionAtom::Subexpression(Self::parse(subexpression)?));
                }


                let base_ident = tokens[0].to_owned();
                // Complex address
                if let Token::Identifier(base_ident) = base_ident {

                    let first_separator = tokens[1].to_owned();

                    // Member of a module
                    if let Token::Punctuation(PunctuationToken::DoubleColon) = first_separator {
                        let mut tokens = tokens.into_iter().skip(2);

                        let member_ident = tokens.next();
                        if let Some(Token::Identifier(member_ident)) = member_ident {
                            match tokens.next() {
                                
                                // Procedure
                                Some(Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Opening))) => {
                                    let arguments = Self::take_until_closing(
                                        &mut tokens,
                                        Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Closing))
                                    )?;

                                    let arguments = Self::split_by_commas(arguments)?;
                                    let mut argument_expressions = Vec::new();
                                    for argument in arguments {
                                        argument_expressions.push(Self::parse(argument)?);
                                    }

                                    let module_address = ModuleAddress::new(base_ident, member_ident);

                                    return Ok(ExpressionAtom::Subexpression(Box::new(ProcedureCallExpression {
                                        procedure_id: module_address,
                                        arguments: argument_expressions
                                    })));
                                }

                                // Struct construction
                                Some(Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening))) => {
                                    let fields = Self::take_until_closing(
                                        &mut tokens,
                                        Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Closing))
                                    )?;
                                    let fields = Self::split_by_commas(fields)?;

                                    let mut field_overrides = Vec::new();

                                    for field in fields {
                                        let mut field = field.into_iter();
                                        let field_ident = field.next();
                                        if let Some(Token::Identifier(field_ident)) = field_ident {
                                            let separator = field.next();
                                            if let Some(Token::Punctuation(PunctuationToken::Colon)) = separator {
                                                field_overrides.push((
                                                    field_ident,
                                                    Self::parse(field)?
                                                ));
                                            } else {
                                                return Err(CompilerError {
                                                    message: format!("Unexpected token. Expected identifier, found {:?}!", separator)
                                                });
                                            }
                                        } else {
                                            return Err(CompilerError {
                                                message: format!("Unexpected token. Expected identifier, found {:?}!", field_ident)
                                            });
                                        }
                                    }

                                    let module_address = ModuleAddress::new(base_ident, member_ident);

                                    return Ok(ExpressionAtom::Subexpression(Box::new(StructConstructionExpression {
                                        struct_id: module_address,
                                        field_overrides
                                    })));
                                }

                                other => {
                                    return Err(CompilerError {
                                        message: format!("Unexpected token: {:?}", other)
                                    });
                                }
                            }
                        } else {
                            return Err(CompilerError {
                                message: format!("Unexpected token. Expected identifier, found {:?}", member_ident)
                            });
                        }
                    } else {
                        return Self::parse_variable_address(tokens);
                    }
                } else {
                    return Err(CompilerError {
                        message: format!("Unexpected token. Expected identifier, found {:?}!", base_ident)
                    });
                }
            },
        }

    }

    fn parse_variable_address(tokens: impl IntoIterator<Item = Token>) -> Result<ExpressionAtom, CompilerError> {

        let mut address = Vec::new();

        let mut tokens = tokens.into_iter();

        while let Some(next) = tokens.next() {
            match next {
                Token::Identifier(ident) => {
                    address.push(ScopeAddressant::Identifier(ident));
                }
                Token::Punctuation(PunctuationToken::Dot) => {}
                Token::Punctuation(PunctuationToken::SquareBrackets(ParenthesisType::Opening)) => {
                    let inner = Self::take_until_closing(
                        &mut tokens,
                        Token::Punctuation(PunctuationToken::SquareBrackets(ParenthesisType::Closing))
                    )?;

                    let index_expression = Self::parse(inner)?;

                    address.push(ScopeAddressant::DynamicIndex(index_expression.into()));
                }

                _ => Err(CompilerError {
                    message: format!("Unexpected token. Expected addressant, found {:?}!", next)
                })?
            }
        }


        Ok(ExpressionAtom::Subexpression(Box::new(VariableExpression {
            variable_address: address.try_into().map_err(|_| CompilerError {
                message: "Could not resolve variable's address!".into()
            })?
        })))
    }

    fn get_precedence(operator: &OperatorToken) -> usize {
        match operator {
            OperatorToken::Assignment => 0,
            OperatorToken::Plus => 1,
            OperatorToken::Minus => 1,
            OperatorToken::Multiply => 2,
            OperatorToken::Divide => 2,
            OperatorToken::Modulo => 3,
            OperatorToken::Power => 4,
            OperatorToken::Not => 10,
            OperatorToken::And => 2,
            OperatorToken::Or => 1,
            OperatorToken::Equality => 0,
            OperatorToken::Inequality => 0,
            OperatorToken::Greater => 0,
            OperatorToken::Less => 0,
            OperatorToken::GreaterEquals => 0,
            OperatorToken::LessEquals => 0,
        }
    }

    fn resolve_binary_operator(
        operator: &OperatorToken,
        lhs: Box<dyn Expression>,
        rhs: Box<dyn Expression>
    ) -> Result<Box<dyn Expression>, CompilerError> {
        match operator {
            OperatorToken::Assignment => Err(CompilerError {
                message: "Assignment operator disallowed in expressions!".into()
            }),
            OperatorToken::Plus => Ok(Box::new(AddExpression::new(lhs, rhs))),
            OperatorToken::Minus => Ok(Box::new(SubtractExpression::new(lhs, rhs))),
            OperatorToken::Multiply => Ok(Box::new(MultiplyExpression::new(lhs, rhs))),
            OperatorToken::Divide => Ok(Box::new(DivideExpression::new(lhs, rhs))),
            OperatorToken::Modulo => Ok(Box::new(ModuloExpression::new(lhs, rhs))),
            OperatorToken::Power => Ok(Box::new(PowerExpression::new(lhs, rhs))),
            OperatorToken::And => Ok(Box::new(AndExpression::new(lhs, rhs))),
            OperatorToken::Or => Ok(Box::new(OrExpression::new(lhs, rhs))),
            OperatorToken::Equality => Ok(Box::new(EqualityExpression::new(lhs, rhs))),
            OperatorToken::Inequality => Ok(Box::new(NotExpression::new(Box::new(EqualityExpression::new(lhs, rhs))))),
            OperatorToken::Not => Err(CompilerError {
                message: "'Not' operator is not a binary operator!".into()
            }),
            OperatorToken::Greater => Ok(Box::new(GreaterThanExpression::new(lhs, rhs))),
            OperatorToken::Less => Ok(Box::new(GreaterThanExpression::new(rhs, lhs))),
            OperatorToken::GreaterEquals => Ok(Box::new(
                NotExpression::new(Box::new(GreaterThanExpression::new(rhs, lhs)))
            )),
            OperatorToken::LessEquals => Ok(Box::new(
                NotExpression::new(Box::new(GreaterThanExpression::new(lhs, rhs)))
            )),
        }
    }
    
}