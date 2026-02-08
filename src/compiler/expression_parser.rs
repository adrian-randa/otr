use std::{collections::HashMap, rc::Rc};

use crate::{compiler::CompilerError, lexer::token::{KeywordToken, OperatorToken, ParenthesisType, PunctuationToken, Token}, runtime::{Expression, ModuleAddress,Type, Value, expressions::{ArrayIndexExpression, AssociatedProcedureCallExpression, CloneExpression, EqualityExpression, ProcedureCallExpression, ReferenceExpression, StructConstructionExpression, StructMemberExpression, TypeofVariableExpression, VariableExpression, arithmetic::{AddExpression, DivideExpression, GreaterThanExpression, ModuloExpression, MultiplyExpression, PowerExpression, SubtractExpression}, boolean::{AndExpression, NotExpression, OrExpression}}, scope::{ScopeAddress, ScopeAddressant}}};

use crate::error::Result;

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
    pub fn parse(expression: impl IntoIterator<Item = Token>) -> Result<Box<dyn Expression>> {
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
                            return Err(CompilerError::InvalidExpression { message: "Expressions may not start with a binary operator!".into() }.boxed());
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
                Err(CompilerError::InvalidExpression {
                    message: "Missing operator!".into()
                }.boxed())?;
            }
        }

        Ok(atoms[0].take().unwrap().unwrap_subexpression())
    }

    pub fn atomize(expression: impl IntoIterator<Item = Token>) -> Result<Vec<ExpressionAtom>> {
        let raw_atoms = Self::split(expression)?;

        let mut atoms = Vec::new();

        for atom in raw_atoms {
           atoms.push(Self::parse_raw_atom(atom)?);
        }

        Ok(atoms)
    }

    pub fn take_until_closing(tokens: impl IntoIterator<Item = Token>, parenthesis: Token) -> Result<Vec<Token>> {
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
                                    let top = stack.pop().ok_or(CompilerError::InvalidParenthesisStructure.boxed())?;

                                    match (&top, &punct) {
                                        (Parenthesis(_), Parenthesis(_)) |
                                        (SquareBrackets(_), SquareBrackets(_)) |
                                        (CurlyBraces(_), CurlyBraces(_)) => {}
                                        _ => {
                                            return Err(CompilerError::InvalidParenthesisStructure.boxed());
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
            return Err(CompilerError::InvalidParenthesisStructure.boxed());
        }

        Ok(slice)
    }


    pub fn split_by_commas(tokens: impl IntoIterator<Item = Token>) -> Result<Vec<Vec<Token>>> {

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
                                let top = stack.pop().ok_or(CompilerError::InvalidParenthesisStructure.boxed())?;

                                match (&top, &punct) {
                                    (Parenthesis(_), Parenthesis(_)) |
                                    (SquareBrackets(_), SquareBrackets(_)) |
                                    (CurlyBraces(_), CurlyBraces(_)) => {}
                                    _ => {
                                        return Err(CompilerError::InvalidParenthesisStructure.boxed());
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

    pub fn split(tokens: impl IntoIterator<Item = Token>) -> Result<Vec<RawExpressionAtom>> {
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
                                    let top = stack.pop().ok_or(CompilerError::InvalidParenthesisStructure.boxed())?;

                                    match (&top, &punct) {
                                        (Parenthesis(_), Parenthesis(_)) |
                                        (SquareBrackets(_), SquareBrackets(_)) |
                                        (CurlyBraces(_), CurlyBraces(_)) => {}
                                        _ => {
                                            return Err(CompilerError::InvalidParenthesisStructure.boxed());
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

    fn parse_raw_atom(atom: RawExpressionAtom) -> Result<ExpressionAtom> {
        Ok(
            match atom {
                RawExpressionAtom::Subexpression(tokens) => ExpressionAtomParser::new().parse(tokens)?,
                RawExpressionAtom::Operator(operator_token) => ExpressionAtom::Operator(operator_token),
            }  
        )      
    }

    fn old_parse_raw_atom(atom: RawExpressionAtom) -> Result<ExpressionAtom> {
        match atom {
            RawExpressionAtom::Operator(operator) => Ok(ExpressionAtom::Operator(operator)),
            RawExpressionAtom::Subexpression(tokens) => {
                // Epmpty
                if tokens.len() == 0 {
                    return Err(CompilerError::InvalidExpression {
                        message: "Found empty subexpression atom!".into()
                    }.boxed());
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
                                    .map_err(|_| CompilerError::InvalidExpression {
                                        message: format!("Could not resolve identifier '{}'!", ident)
                                    }.boxed())?
                            })))
                        }
                        _ => {
                            return Err(CompilerError::UnexpectedToken { expected: None, found: token.clone() }.boxed());
                        }
                    }
                }

                // Parenthesis
                if let Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Opening)) = tokens[0] {
                    let mut tokens = tokens.into_iter().skip(1);
                    let subexpression = Self::take_until_closing(
                        &mut tokens,
                        Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Closing))
                    )?;

                    if let Some(token) = tokens.next() {
                        Err(CompilerError::UnexpectedToken { expected: Some("Operator".into()), found: token }.boxed())?;
                    }

                    return Ok(ExpressionAtom::Subexpression(Self::parse(subexpression)?));
                }


                let first_token = tokens[0].to_owned();
                match first_token {
                    Token::Identifier(base_ident) => {
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
                                                    return Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found: separator.unwrap_or(Token::Identifier("".into())) }.boxed());
                                                }
                                            } else {
                                                return Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found: field_ident.unwrap_or(Token::Identifier("".into())) }.boxed());
                                            }
                                        }

                                        let module_address = ModuleAddress::new(base_ident, member_ident);

                                        return Ok(ExpressionAtom::Subexpression(Box::new(StructConstructionExpression {
                                            struct_id: module_address,
                                            field_overrides
                                        })));
                                    }

                                    other => {
                                        return Err(CompilerError::UnexpectedToken { expected: None, found: other.unwrap_or(Token::Identifier("".into())) }.boxed());
                                    }
                                }
                            } else {
                                return Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found: member_ident.unwrap_or(Token::Identifier("".into())) }.boxed());
                            }
                        } else {
                            return Self::parse_variable_address(tokens);
                        }
                    }
                    Token::Keyword(KeywordToken::Ref) => {
                        let mut tokens = tokens;
                        let tokens: Vec<Token> = tokens.drain(1..).collect();

                        let variable_address = ScopeAddress::try_from(tokens)?;

                        Ok(ExpressionAtom::Subexpression(Box::new(ReferenceExpression { variable_address })))
                    }
                    Token::Keyword(KeywordToken::Clone) => {
                        let mut tokens = tokens;
                        let tokens: Vec<Token> = tokens.drain(1..).collect();

                        let variable_address = ScopeAddress::try_from(tokens)?;

                        Ok(ExpressionAtom::Subexpression(Box::new(CloneExpression { variable_address })))
                    }
                    _ => {
                        return Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found: first_token }.boxed());
                    }
                }
            },
        }

    }

    fn parse_variable_address(tokens: impl IntoIterator<Item = Token>) -> Result<ExpressionAtom> {

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

                _ => Err(CompilerError::UnexpectedToken { expected: Some("Addressant".into()), found: next }.boxed())?
            }
        }


        Ok(ExpressionAtom::Subexpression(Box::new(VariableExpression {
            variable_address: address.try_into().map_err(|_| CompilerError::InvalidExpression {
                message: "Could not resolve variable's address!".into()
            }.boxed())?
        })))
    }

    fn get_precedence(operator: &OperatorToken) -> usize {
        match operator {
            OperatorToken::Assignment => 0,
            OperatorToken::Plus => 1,
            OperatorToken::Minus => 1,
            OperatorToken::Multiply => 2,
            OperatorToken::Divide => 2,
            OperatorToken::Modulo => 4,
            OperatorToken::Power => 5,
            OperatorToken::Not => 10,
            OperatorToken::And => 2,
            OperatorToken::Or => 1,
            OperatorToken::Equality => 3,
            OperatorToken::Inequality => 3,
            OperatorToken::Greater => 3,
            OperatorToken::Less => 3,
            OperatorToken::GreaterEquals => 3,
            OperatorToken::LessEquals => 3,
        }
    }

    fn resolve_binary_operator(
        operator: &OperatorToken,
        lhs: Box<dyn Expression>,
        rhs: Box<dyn Expression>
    ) -> Result<Box<dyn Expression>> {
        match operator {
            OperatorToken::Assignment => Err(CompilerError::InvalidExpression {
                message: "Assignment operator disallowed in expressions!".into()
            }.boxed()),
            OperatorToken::Plus => Ok(Box::new(AddExpression::new(lhs, rhs)) as Box<dyn Expression>),
            OperatorToken::Minus => Ok(Box::new(SubtractExpression::new(lhs, rhs)) as Box<dyn Expression>),
            OperatorToken::Multiply => Ok(Box::new(MultiplyExpression::new(lhs, rhs)) as Box<dyn Expression>),
            OperatorToken::Divide => Ok(Box::new(DivideExpression::new(lhs, rhs)) as Box<dyn Expression>),
            OperatorToken::Modulo => Ok(Box::new(ModuloExpression::new(lhs, rhs)) as Box<dyn Expression>),
            OperatorToken::Power => Ok(Box::new(PowerExpression::new(lhs, rhs)) as Box<dyn Expression>),
            OperatorToken::And => Ok(Box::new(AndExpression::new(lhs, rhs)) as Box<dyn Expression>),
            OperatorToken::Or => Ok(Box::new(OrExpression::new(lhs, rhs)) as Box<dyn Expression>),
            OperatorToken::Equality => Ok(Box::new(EqualityExpression::new(lhs, rhs)) as Box<dyn Expression>),
            OperatorToken::Inequality => Ok(Box::new(NotExpression::new(Box::new(EqualityExpression::new(lhs, rhs)))) as Box<dyn Expression>),
            OperatorToken::Not => Err(CompilerError::InvalidExpression {
                message: "'Not' operator is not a binary operator!".into()
            }.boxed()),
            OperatorToken::Greater => Ok(Box::new(GreaterThanExpression::new(lhs, rhs)) as Box<dyn Expression>),
            OperatorToken::Less => Ok(Box::new(GreaterThanExpression::new(rhs, lhs)) as Box<dyn Expression>),
            OperatorToken::GreaterEquals => Ok(Box::new(
                NotExpression::new(Box::new(GreaterThanExpression::new(rhs, lhs)))
            ) as Box<dyn Expression>),
            OperatorToken::LessEquals => Ok(Box::new(
                NotExpression::new(Box::new(GreaterThanExpression::new(lhs, rhs)))
            ) as Box<dyn Expression>),
        }
    }
    
}

#[derive(Debug, PartialEq, Eq)]
enum ScopeAccessMode {
    Move,
    Ref,
    Clone,
    Typeof,
}

enum ExpressionAtomParserState {
    Base,
    SingleIdent {
        ident: String,
    },
    ScopeAddress {
        address: Vec<ScopeAddressant>,
        access: ScopeAccessMode,
    },
    ScopeAddressMember {
        address: Vec<ScopeAddressant>,
        access: ScopeAccessMode,
    },
    Subexpression {
        subexpression: Box<dyn Expression>
    },
    ModuleMember {
        module_ident: String,
        member_ident: Option<String>,
    },
    StructMember {
        subexpression: Box<dyn Expression>,
    },
    AssociatedProcedureCall {
        subexpression: Box<dyn Expression>,
        ident: Option<String>,
    },
}

struct ExpressionAtomParser {
    state: ExpressionAtomParserState,
}

impl ExpressionAtomParser {

    fn new() -> Self {
        Self {
            state: ExpressionAtomParserState::Base,
        }
    }

    fn parse(mut self, tokens: impl IntoIterator<Item = Token>) -> Result<ExpressionAtom> {

        let mut tokens = tokens.into_iter();

        while let Some(token) = tokens.next() {
            use ExpressionAtomParserState::*;
            
            match self.state {
                Base => {
                    match token {
                        Token::Literal(literal) => {
                            self.state = Subexpression { subexpression: Box::new(Value::try_from(literal)?) }
                        }
                        Token::Identifier(ident) => {
                            self.state = SingleIdent { ident };
                        }
                        Token::Keyword(KeywordToken::Ref) => {
                            self.state = ScopeAddressMember {
                                access: ScopeAccessMode::Ref,
                                address: Vec::new()
                            };
                        }
                        Token::Keyword(KeywordToken::Clone) => {
                            self.state = ScopeAddressMember {
                                access: ScopeAccessMode::Clone,
                                address: Vec::new()
                            };
                        }
                        Token::Keyword(KeywordToken::Typeof) => {
                            self.state = ScopeAddressMember {
                                access: ScopeAccessMode::Typeof,
                                address: Vec::new()
                            };
                        }
                        Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Opening)) => {
                            let inner = ExpressionParser::take_until_closing(
                                &mut tokens,
                                Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Closing))
                            )?;
                            self.state = Subexpression { 
                                subexpression: ExpressionParser::parse(inner)?
                            }
                        }
                        other=> {
                            return Err(CompilerError::UnexpectedToken { expected: Some("Literal or Identifier".into()), found: other }.boxed());
                        }
                    }
                },
                SingleIdent { ident } => {
                    match token {
                        Token::Punctuation(PunctuationToken::Dot) => {
                            self.state = ScopeAddressMember {
                                address: vec![
                                    ScopeAddressant::Identifier(ident)
                                ], 
                                access: ScopeAccessMode::Move
                            };
                        }
                        Token::Punctuation(PunctuationToken::SquareBrackets(ParenthesisType::Opening)) => {
                            let inner = ExpressionParser::take_until_closing(
                                &mut tokens,
                                Token::Punctuation(PunctuationToken::SquareBrackets(ParenthesisType::Closing))
                            )?;
                            let index_expression = ExpressionParser::parse(inner)?;

                            self.state = ScopeAddress {
                                address: vec![
                                    ScopeAddressant::Identifier(ident),
                                    ScopeAddressant::DynamicIndex(index_expression.into())
                                ],
                                access: ScopeAccessMode::Move
                            };
                        }
                        Token::Punctuation(PunctuationToken::DoubleColon) => {
                            self.state = ModuleMember {
                                module_ident: ident,
                                member_ident: None,
                            }
                        }
                        Token::Punctuation(PunctuationToken::ThinArrow) => {
                            self.state = AssociatedProcedureCall {
                                subexpression: Box::new(VariableExpression {
                                    variable_address: vec![
                                        ScopeAddressant::Identifier(ident)
                                    ].try_into().unwrap()
                                }),
                                ident: None,
                            }
                        }
                        other => {
                            return Err(CompilerError::UnexpectedToken { expected: Some("Scope Address, Module Member or Associated Procedure Call".into()), found: other }.boxed());
                        }
                    }
                },
                ScopeAddress { mut address, access } => {
                    match token {
                        Token::Punctuation(PunctuationToken::Dot) => {
                            self.state = ScopeAddressMember { address, access };
                        }
                        Token::Punctuation(PunctuationToken::SquareBrackets(ParenthesisType::Opening)) => {
                            let inner = ExpressionParser::take_until_closing(
                                &mut tokens,
                                Token::Punctuation(PunctuationToken::SquareBrackets(ParenthesisType::Closing))
                            )?;
                            let index_expression = ExpressionParser::parse(inner)?;

                            address.push(ScopeAddressant::DynamicIndex(index_expression.into()));

                            self.state = ScopeAddress { address, access };
                        }
                        Token::Punctuation(PunctuationToken::ThinArrow) => {
                            let variable_address = address.try_into().unwrap();
                            let subexpression = match access {
                                ScopeAccessMode::Move => Box::new(VariableExpression { variable_address }) as Box<dyn Expression>,
                                ScopeAccessMode::Ref => Box::new(ReferenceExpression { variable_address }) as Box<dyn Expression>,
                                ScopeAccessMode::Clone => Box::new(CloneExpression { variable_address }) as Box<dyn Expression>,
                                ScopeAccessMode::Typeof => Box::new(TypeofVariableExpression { variable_address }) as Box<dyn Expression>,
                            };

                            self.state = AssociatedProcedureCall { subexpression, ident: None };
                        }
                        other => {
                            return Err(CompilerError::UnexpectedToken { expected: Some(". or index".into()), found: other }.boxed());
                        }
                    }
                },
                ScopeAddressMember { mut address, access } => {
                    match token {
                        Token::Identifier(ident) => {
                            address.push(ScopeAddressant::Identifier(ident));
                            self.state = ScopeAddress { address, access };
                        }

                        other => {
                            return Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found: other }.boxed());
                        }
                    }
                }
                Subexpression { subexpression } => {
                    match token {
                        Token::Punctuation(PunctuationToken::Dot) => {
                            self.state = StructMember { subexpression };
                        }
                        Token::Punctuation(PunctuationToken::SquareBrackets(ParenthesisType::Opening)) => {
                            let inner = ExpressionParser::take_until_closing(
                                &mut tokens,
                                Token::Punctuation(PunctuationToken::SquareBrackets(ParenthesisType::Closing))
                            )?;
                            let index_expression = ExpressionParser::parse(inner)?;

                            self.state = Subexpression {
                                subexpression: Box::new(ArrayIndexExpression { subexpression, index_expression })
                            };
                        }
                        Token::Punctuation(PunctuationToken::ThinArrow) => {
                            self.state = AssociatedProcedureCall {
                                subexpression,
                                ident: None,
                            }
                        }
                        other => {
                            return Err(CompilerError::UnexpectedToken { expected: Some("Scope Address, Module Member or Associated Procedure Call".into()), found: other }.boxed());
                        }
                    }
                },
                ModuleMember { module_ident, member_ident } => {
                    if let Some(member_ident) = member_ident {
                        match token {
                            Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Opening)) => {
                                let inner = ExpressionParser::take_until_closing(
                                    &mut tokens,
                                    Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Closing))
                                )?;
                                let raw_args = ExpressionParser::split_by_commas(inner)?;
                                let mut arguments = Vec::new();
                                for arg in raw_args {
                                    arguments.push(ExpressionParser::parse(arg)?);
                                }

                                self.state = Subexpression {
                                    subexpression: Box::new(ProcedureCallExpression {
                                        procedure_id: ModuleAddress::new(module_ident, member_ident),
                                        arguments
                                    })
                                };
                            }
                            Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) => {
                                let inner = ExpressionParser::take_until_closing(
                                    &mut tokens,
                                    Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Closing))
                                )?;
                                let raw_field_overrides = ExpressionParser::split_by_commas(inner)?;
                                let mut field_overrides = Vec::new();
                                for field_override in raw_field_overrides {
                                    let mut tokens = field_override.into_iter();

                                    let field_name = tokens.next();
                                    let field_name = match field_name {
                                        Some(Token::Identifier(ident)) => {
                                            ident
                                        },
                                        other => {
                                            return Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found: other.unwrap_or(Token::Identifier("".into())) }.boxed());
                                        }
                                    };

                                    match tokens.next() {
                                        Some(Token::Punctuation(PunctuationToken::Colon)) => {},
                                        other => {
                                            return Err(CompilerError::UnexpectedToken { expected: Some(":".into()), found: other.unwrap_or(Token::Identifier("".into())) }.boxed());
                                        }
                                    };

                                    field_overrides.push((
                                        field_name,
                                        ExpressionParser::parse(tokens)?
                                    ));
                                }

                                self.state = Subexpression {
                                    subexpression: Box::new(StructConstructionExpression {
                                        struct_id: ModuleAddress::new(module_ident, member_ident),
                                        field_overrides,
                                    })
                                };
                            }

                            other => {
                                return Err(CompilerError::UnexpectedToken { expected: Some("Procedure Call or Struct Construction".into()), found: other }.boxed());
                            }
                        }
                    } else {
                        if let Token::Identifier(member_ident) = token {
                            self.state = ModuleMember { module_ident, member_ident: Some(member_ident) }
                        } else {
                            return Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found: token }.boxed());
                        }
                    }
                },
                StructMember { subexpression } => {
                    match token {
                        Token::Identifier(member_ident) => {
                            self.state = Subexpression { 
                                subexpression: Box::new(StructMemberExpression {
                                    subexpression,
                                    member_ident,
                                })
                             }
                        }

                        other => {
                            return Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found: other }.boxed());
                        }
                    }
                },
                AssociatedProcedureCall { subexpression, ident } => {
                    if let Some(ident) = ident {
                        match token {
                            Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Opening)) => {
                                let inner = ExpressionParser::take_until_closing(
                                    &mut tokens,
                                    Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Closing))
                                )?;
                                let raw_args = ExpressionParser::split_by_commas(inner)?;
                                let mut arguments = Vec::new();
                                for arg in raw_args {
                                    arguments.push(ExpressionParser::parse(arg)?);
                                }

                                self.state = Subexpression {
                                    subexpression: Box::new(AssociatedProcedureCallExpression {
                                        callee_expression: subexpression,
                                        procedure_ident: ident,
                                        arguments,
                                    })
                                };
                            }

                            other => {
                                return Err(CompilerError::UnexpectedToken { expected: Some("(".into()), found: other }.boxed());
                            }
                        }
                    } else {
                        match token {
                            Token::Identifier(ident) => {
                                self.state = AssociatedProcedureCall { subexpression, ident: Some(ident) };
                            }

                            other => {
                                return Err(CompilerError::UnexpectedToken { expected: Some("Identifier".into()), found: other }.boxed());
                            }
                        }
                    }
                },
            }
        }

        match self.state {
            ExpressionAtomParserState::Base => Err(CompilerError::InvalidExpression {
                message: "Empty subexpression atom!".into()
            }.boxed()),
            ExpressionAtomParserState::SingleIdent { ident } => Ok(ExpressionAtom::Subexpression(
                Box::new(VariableExpression {
                    variable_address: vec![ScopeAddressant::Identifier(ident)].try_into().unwrap()
                })
            )),
            ExpressionAtomParserState::ScopeAddress { address, access } => Ok(
                ExpressionAtom::Subexpression({
                        let variable_address = address.try_into().unwrap();
                        match access {
                            ScopeAccessMode::Move => Box::new(VariableExpression { variable_address }) as Box<dyn Expression>,
                            ScopeAccessMode::Ref => Box::new(ReferenceExpression { variable_address }) as Box<dyn Expression>,
                            ScopeAccessMode::Clone => Box::new(CloneExpression { variable_address }) as Box<dyn Expression>,
                            ScopeAccessMode::Typeof => Box::new(TypeofVariableExpression { variable_address }) as Box<dyn Expression>,
                        }
                    })
                ),
            ExpressionAtomParserState::ScopeAddressMember { address, access } => Err(
                CompilerError::InvalidExpression { message: "Missing token. Expected identifier after '.'!".into() }.boxed()
            ),
            ExpressionAtomParserState::Subexpression { subexpression } => Ok(
                ExpressionAtom::Subexpression(subexpression)
            ),
            ExpressionAtomParserState::ModuleMember { module_ident, member_ident } => {
                if let Some(member_ident) = member_ident {
                    Ok(ExpressionAtom::Subexpression(
                        Box::new(Value::Type(
                            Type::Struct { struct_id: ModuleAddress::new(module_ident, member_ident) }
                        ))
                    ))
                } else {
                    Err(
                        CompilerError::InvalidExpression {
                            message: "Incomplete subexpression!".into()
                        }.boxed()
                    )
                }
            }
            
            ExpressionAtomParserState::StructMember { subexpression } => Err(
                CompilerError::InvalidExpression {
                    message: "Missing token. Expected identifier after '.'!".into()
                }.boxed()
            ),
            ExpressionAtomParserState::AssociatedProcedureCall { subexpression, ident } => Err(CompilerError::InvalidExpression {
                message: "Incomplete associated procedure call!".into()
            }.boxed()),
        }
    }
}