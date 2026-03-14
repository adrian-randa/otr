use crate::{
    compiler::{CompilerError, ExpressionParseEnvironment, parenthesis::ParenthesisStack}, core::{expression::{ArrayConstructionExpression, ArrayIndexExpression, AssociatedProcedureCallExpression, CatchExpression, Expression, ProcedureCallExpression, StructConstructionExpression, StructMemberExpression, arithmetic::ArithmeticExpression, boolean::BooleanExpression, comparison::ComparisonExpression, variable::{VariableAccessMode, VariableAddressant, VariableExpression}}, module::ModuleAddress, r#type::Type, value::Value}, lexer::token::{KeywordToken, LiteralToken, OperatorToken, ParenthesisType, PrimitiveTypeToken, PunctuationToken, Token}
};

use crate::error::Result;

#[derive(Debug)]
pub enum ExpressionAtom {
    Subexpression(Expression),
    Operator(OperatorToken),
}

impl ExpressionAtom {
    fn unwrap_subexpression(self) -> Expression {
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
    pub fn parse(
        expression: impl IntoIterator<Item = Token>,
        environment: &dyn ExpressionParseEnvironment,
    ) -> Result<Expression> {
        let atoms = Self::atomize(expression, environment)?;

        let mut operator_order = Vec::new();
        for i in 0..atoms.len() {
            if let ExpressionAtom::Operator(operator) = &atoms[i] {
                operator_order.push((Self::get_precedence(operator), i));
            }
        }
        operator_order.sort_by_key(|(precedence, _i)| usize::MAX - *precedence);

        let mut atoms = atoms.into_iter().map(|atom| Some(atom)).collect::<Vec<_>>();

        for i in 0..operator_order.len() {
            if let Some(ExpressionAtom::Operator(op)) = atoms[operator_order[i].1].take() {
                match op {
                    OperatorToken::Not => {
                        if let Some(ExpressionAtom::Subexpression(subexpr)) =
                            atoms[operator_order[i].1 + 1].take()
                        {
                            let splice = vec![Some(ExpressionAtom::Subexpression(
                                Expression::Boolean(BooleanExpression::Not(Box::new(subexpr)))
                            ))];

                            atoms.splice(i..=i + 1, splice);

                            for operator in &mut operator_order {
                                if operator.1 > i {
                                    *operator = (operator.0, operator.1 - 2);
                                }
                            }
                        }
                    }

                    op => {
                        if operator_order[i].1 == 0 {
                            return Err(CompilerError::InvalidExpression {
                                message: "Expressions may not start with a binary operator!".into(),
                            }
                            .boxed());
                        }
                        if let (
                            Some(ExpressionAtom::Subexpression(lhs)),
                            Some(ExpressionAtom::Subexpression(rhs)),
                        ) = (
                            atoms[operator_order[i].1 - 1].take(),
                            atoms[operator_order[i].1 + 1].take(),
                        ) {
                            let splice = vec![Some(ExpressionAtom::Subexpression(
                                Self::resolve_binary_operator(&op, lhs, rhs)?,
                            ))];
                            let op_index = operator_order[i].1;

                            atoms.splice(
                                (operator_order[i].1 - 1)..=(operator_order[i].1 + 1),
                                splice,
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
                    message: "Missing operator!".into(),
                }
                .boxed())?;
            }
        }

        Ok(atoms[0].take().unwrap().unwrap_subexpression())
    }

    pub fn atomize(
        expression: impl IntoIterator<Item = Token>,
        environment: &dyn ExpressionParseEnvironment,
    ) -> Result<Vec<ExpressionAtom>> {
        let raw_atoms = Self::split(expression)?;

        let mut atoms = Vec::new();

        for atom in raw_atoms {
            atoms.push(Self::parse_raw_atom(atom, environment)?);
        }

        Ok(atoms)
    }

    pub fn take_until_closing(
        tokens: impl IntoIterator<Item = Token>,
        parenthesis: Token,
    ) -> Result<Vec<Token>> {
        use PunctuationToken::*;

        let mut stack = ParenthesisStack::new();

        match parenthesis {
            Token::Punctuation(PunctuationToken::Parenthesis(_)) => {
                stack.read(Token::Punctuation(Parenthesis(ParenthesisType::Opening)))?;
            }
            Token::Punctuation(PunctuationToken::SquareBrackets(_)) => {
                stack.read(Token::Punctuation(SquareBrackets(ParenthesisType::Opening)))?;
            }
            Token::Punctuation(PunctuationToken::CurlyBraces(_)) => {
                stack.read(Token::Punctuation(CurlyBraces(ParenthesisType::Opening)))?;
            }

            _ => panic!("Unsupported parenthesis type!"),
        };

        let mut slice = Vec::new();

        let mut iter = tokens.into_iter();

        while let Some(token) = iter.next() {
            if stack.len() == 1 && &token == &parenthesis {
                return Ok(slice);
            }
            stack.read(token.clone())?;
            slice.push(token);
        }

        if !stack.is_empty() {
            return Err(CompilerError::InvalidParenthesisStructure.boxed());
        }

        Ok(slice)
    }

    fn split_by_commas(tokens: impl IntoIterator<Item = Token>) -> Result<Vec<Vec<Token>>> {
        let mut iter = tokens.into_iter();

        let mut slices = Vec::new();

        let mut current = Vec::new();

        let mut stack = Vec::new();

        while let Some(next) = iter.next() {
            if let Token::Punctuation(punct) = next.clone() {
                use PunctuationToken::*;

                match &punct {
                    Parenthesis(p) | SquareBrackets(p) | CurlyBraces(p) => match p {
                        ParenthesisType::Opening => stack.push(punct),
                        ParenthesisType::Closing => {
                            let top = stack
                                .pop()
                                .ok_or(CompilerError::InvalidParenthesisStructure.boxed())?;

                            match (&top, &punct) {
                                (Parenthesis(_), Parenthesis(_))
                                | (SquareBrackets(_), SquareBrackets(_))
                                | (CurlyBraces(_), CurlyBraces(_)) => {}
                                _ => {
                                    return Err(CompilerError::InvalidParenthesisStructure.boxed());
                                }
                            }
                        }
                    },

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

    fn split(tokens: impl IntoIterator<Item = Token>) -> Result<Vec<RawExpressionAtom>> {
        let mut tokens = tokens.into_iter();

        let mut atoms = Vec::new();
        let mut current_subexpression = Vec::new();

        let mut stack = Vec::new();

        while let Some(next) = tokens.next() {
            match next.clone() {
                Token::Punctuation(punct) => {
                    use PunctuationToken::*;

                    match &punct {
                        Parenthesis(p) | SquareBrackets(p) | CurlyBraces(p) => match p {
                            ParenthesisType::Opening => stack.push(punct),
                            ParenthesisType::Closing => {
                                let top = stack
                                    .pop()
                                    .ok_or(CompilerError::InvalidParenthesisStructure.boxed())?;

                                match (&top, &punct) {
                                    (Parenthesis(_), Parenthesis(_))
                                    | (SquareBrackets(_), SquareBrackets(_))
                                    | (CurlyBraces(_), CurlyBraces(_)) => {}
                                    _ => {
                                        return Err(
                                            CompilerError::InvalidParenthesisStructure.boxed()
                                        );
                                    }
                                }
                            }
                        },

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

    fn parse_raw_atom(
        atom: RawExpressionAtom,
        environment: &dyn ExpressionParseEnvironment,
    ) -> Result<ExpressionAtom> {
        Ok(match atom {
            RawExpressionAtom::Subexpression(tokens) => {
                ExpressionAtomParser::new().parse(tokens, environment)?
            }
            RawExpressionAtom::Operator(operator_token) => ExpressionAtom::Operator(operator_token),
        })
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
        lhs: Expression,
        rhs: Expression,
    ) -> Result<Expression> {
        let lhs = Box::new(lhs);
        let rhs = Box::new(rhs);

        match operator {
            OperatorToken::Assignment => Err(CompilerError::InvalidExpression {
                message: "Assignment operator disallowed in expressions!".into(),
            }
            .boxed()),
            OperatorToken::Plus => {
                Ok(Expression::Arithmetic(ArithmeticExpression::Add { lhs, rhs }))
            }
            OperatorToken::Minus => {
                Ok(Expression::Arithmetic(ArithmeticExpression::Subtract { lhs, rhs }))
            }
            OperatorToken::Multiply => {
                Ok(Expression::Arithmetic(ArithmeticExpression::Multiply { lhs, rhs }))
            }
            OperatorToken::Divide => {
                Ok(Expression::Arithmetic(ArithmeticExpression::Divide { lhs, rhs }))
            }
            OperatorToken::Modulo => {
                Ok(Expression::Arithmetic(ArithmeticExpression::Modulo { lhs, rhs }))
            }
            OperatorToken::Power => {
                Ok(Expression::Arithmetic(ArithmeticExpression::Power { base: lhs, exponent: rhs }))
            }
            OperatorToken::And => Ok(Expression::Boolean(BooleanExpression::And { lhs, rhs })),
            OperatorToken::Or => Ok(Expression::Boolean(BooleanExpression::Or { lhs, rhs })),
            OperatorToken::Equality => {
                Ok(Expression::Comparison(ComparisonExpression::Equals { lhs, rhs }))
            }
            OperatorToken::Inequality => Ok(
                Expression::Boolean(BooleanExpression::Not(
                    Box::new(Expression::Comparison(ComparisonExpression::Equals { lhs, rhs }))
                ))
            ),
            OperatorToken::Not => Err(CompilerError::InvalidExpression {
                message: "'Not' operator is not a binary operator!".into(),
            }
            .boxed()),
            OperatorToken::Greater => {
                Ok(Expression::Comparison(ComparisonExpression::GreaterThan { lhs, rhs }))
            }
            OperatorToken::Less => {
                Ok(Expression::Comparison(ComparisonExpression::GreaterThan { lhs: rhs, rhs: lhs }))
            }
            OperatorToken::GreaterEquals => Ok(
                Expression::Boolean(BooleanExpression::Not(
                    Box::new(Expression::Comparison(ComparisonExpression::GreaterThan { lhs: rhs, rhs: lhs }))
                ))
            ),
            OperatorToken::LessEquals => Ok(
                Expression::Boolean(BooleanExpression::Not(
                    Box::new(Expression::Comparison(ComparisonExpression::GreaterThan { lhs, rhs }))
                ))
            ),
        }
    }
}

enum ExpressionAtomParserState {
    Base,
    SingleIdent {
        ident: String,
    },
    ScopeAddress {
        address: Vec<VariableAddressant>,
        access: VariableAccessMode,
    },
    ScopeAddressMember {
        address: Vec<VariableAddressant>,
        access: VariableAccessMode,
    },
    Subexpression {
        subexpression: Expression,
    },
    ModuleMember {
        module_ident: String,
        member_ident: Option<String>,
    },
    StructMember {
        subexpression: Expression,
    },
    AssociatedProcedureCall {
        subexpression: Expression,
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

    fn parse(
        mut self,
        tokens: impl IntoIterator<Item = Token>,
        environment: &dyn ExpressionParseEnvironment,
    ) -> Result<ExpressionAtom> {
        let mut tokens = tokens.into_iter();

        while let Some(token) = tokens.next() {
            use ExpressionAtomParserState::*;

            match self.state {
                Base => match token {
                    Token::Keyword(KeywordToken::Catch) => {
                        self.state = Subexpression {
                            subexpression: Expression::Catch(CatchExpression::new(
                                Box::new(ExpressionParser::parse(&mut tokens, environment)?)
                            ))
                        };
                    }

                    Token::Literal(literal) => {
                        self.state = Subexpression {
                            subexpression: Expression::Value(try_parse_literal(literal)?),
                        }
                    }
                    Token::Identifier(ident) => {
                        self.state = SingleIdent { ident };
                    }
                    Token::Keyword(KeywordToken::Ref) => {
                        self.state = ScopeAddressMember {
                            access: VariableAccessMode::Ref,
                            address: Vec::new(),
                        };
                    }
                    Token::Keyword(KeywordToken::Clone) => {
                        self.state = ScopeAddressMember {
                            access: VariableAccessMode::Clone,
                            address: Vec::new(),
                        };
                    }
                    Token::Keyword(KeywordToken::Typeof) => {
                        self.state = ScopeAddressMember {
                            access: VariableAccessMode::TypeOf,
                            address: Vec::new(),
                        };
                    }
                    Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Opening)) => {
                        let inner = ExpressionParser::take_until_closing(
                            &mut tokens,
                            Token::Punctuation(PunctuationToken::Parenthesis(
                                ParenthesisType::Closing,
                            )),
                        )?;
                        self.state = Subexpression {
                            subexpression: ExpressionParser::parse(inner, environment)?,
                        }
                    }
                    Token::Punctuation(PunctuationToken::SquareBrackets(
                        ParenthesisType::Opening,
                    )) => {
                        let inner = ExpressionParser::take_until_closing(
                            &mut tokens,
                            Token::Punctuation(PunctuationToken::SquareBrackets(
                                ParenthesisType::Closing,
                            )),
                        )?;
                        let raw_items = ExpressionParser::split_by_commas(inner)?;
                        let mut items = Vec::with_capacity(raw_items.len());
                        for raw_item in raw_items {
                            items.push(ExpressionParser::parse(raw_item, environment)?);
                        }
                        self.state = Subexpression {
                            subexpression: Expression::ArrayConstruction(ArrayConstructionExpression::new(items)),
                        }
                    }
                    other => {
                        return Err(CompilerError::UnexpectedToken {
                            expected: Some("Literal or Identifier".into()),
                            found: other,
                        }
                        .boxed());
                    }
                },
                SingleIdent { ident } => match token {
                    Token::Punctuation(PunctuationToken::Dot) => {
                        self.state = ScopeAddressMember {
                            address: vec![VariableAddressant::Identifier(ident)],
                            access: VariableAccessMode::Move,
                        };
                    }
                    Token::Punctuation(PunctuationToken::SquareBrackets(
                        ParenthesisType::Opening,
                    )) => {
                        let inner = ExpressionParser::take_until_closing(
                            &mut tokens,
                            Token::Punctuation(PunctuationToken::SquareBrackets(
                                ParenthesisType::Closing,
                            )),
                        )?;
                        let index_expression = ExpressionParser::parse(inner, environment)?;

                        self.state = ScopeAddress {
                            address: vec![
                                VariableAddressant::Identifier(ident),
                                VariableAddressant::DynamicIndex(index_expression.into()),
                            ],
                            access: VariableAccessMode::Move,
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
                            subexpression: Expression::Variable(VariableExpression::new(
                                vec![VariableAddressant::Identifier(ident)]
                                    .try_into()
                                    .unwrap(),
                                VariableAccessMode::Move
                            )),
                            ident: None,
                        }
                    }

                    Token::Punctuation(PunctuationToken::Parenthesis(ParenthesisType::Opening)) => {
                        let module_address = environment.resolve_procedure_identifier(ident)?;

                        let inner = ExpressionParser::take_until_closing(
                            &mut tokens,
                            Token::Punctuation(PunctuationToken::Parenthesis(
                                ParenthesisType::Closing,
                            )),
                        )?;
                        let raw_args = ExpressionParser::split_by_commas(inner)?;
                        let mut arguments = Vec::new();
                        for arg in raw_args {
                            arguments.push(ExpressionParser::parse(arg, environment)?);
                        }

                        self.state = Subexpression {
                            subexpression: Expression::ProcedureCall(ProcedureCallExpression::new(
                                module_address,
                                arguments,
                            )),
                        };
                    }

                    Token::Punctuation(PunctuationToken::CurlyBraces(ParenthesisType::Opening)) => {
                        let module_address = environment.resolve_struct_identifier(ident)?;

                        let inner = ExpressionParser::take_until_closing(
                            &mut tokens,
                            Token::Punctuation(PunctuationToken::CurlyBraces(
                                ParenthesisType::Closing,
                            )),
                        )?;
                        let raw_field_overrides = ExpressionParser::split_by_commas(inner)?;
                        let mut field_overrides = Vec::new();
                        for field_override in raw_field_overrides {
                            let mut tokens = field_override.into_iter();

                            let field_name = tokens.next();
                            let field_name = match field_name {
                                Some(Token::Identifier(ident)) => ident,
                                other => {
                                    return Err(CompilerError::UnexpectedToken {
                                        expected: Some("Identifier".into()),
                                        found: other.unwrap_or(Token::Identifier("".into())),
                                    }
                                    .boxed());
                                }
                            };

                            match tokens.next() {
                                Some(Token::Punctuation(PunctuationToken::Colon)) => {}
                                other => {
                                    return Err(CompilerError::UnexpectedToken {
                                        expected: Some(":".into()),
                                        found: other.unwrap_or(Token::Identifier("".into())),
                                    }
                                    .boxed());
                                }
                            };

                            field_overrides
                                .push((field_name, ExpressionParser::parse(tokens, environment)?));
                        }

                        self.state = Subexpression {
                            subexpression: Expression::StructConstruction(StructConstructionExpression::new(
                                module_address,
                                field_overrides,
                            )),
                        };
                    }

                    other => {
                        return Err(CompilerError::UnexpectedToken {
                            expected: Some(
                                "Scope Address, Module Member or Associated Procedure Call".into(),
                            ),
                            found: other,
                        }
                        .boxed());
                    }
                },
                ScopeAddress {
                    mut address,
                    access,
                } => match token {
                    Token::Punctuation(PunctuationToken::Dot) => {
                        self.state = ScopeAddressMember { address, access };
                    }
                    Token::Punctuation(PunctuationToken::SquareBrackets(
                        ParenthesisType::Opening,
                    )) => {
                        let inner = ExpressionParser::take_until_closing(
                            &mut tokens,
                            Token::Punctuation(PunctuationToken::SquareBrackets(
                                ParenthesisType::Closing,
                            )),
                        )?;
                        let index_expression = ExpressionParser::parse(inner, environment)?;

                        address.push(VariableAddressant::DynamicIndex(index_expression.into()));

                        self.state = ScopeAddress { address, access };
                    }
                    Token::Punctuation(PunctuationToken::ThinArrow) => {
                        let subexpression = Expression::Variable(VariableExpression::new(
                            address.try_into().unwrap(),
                            access
                        ));

                        self.state = AssociatedProcedureCall {
                            subexpression,
                            ident: None,
                        };
                    }
                    other => {
                        return Err(CompilerError::UnexpectedToken {
                            expected: Some(". or index".into()),
                            found: other,
                        }
                        .boxed());
                    }
                },
                ScopeAddressMember {
                    mut address,
                    access,
                } => match token {
                    Token::Identifier(ident) => {
                        address.push(VariableAddressant::Identifier(ident));
                        self.state = ScopeAddress { address, access };
                    }

                    other => {
                        return Err(CompilerError::UnexpectedToken {
                            expected: Some("Identifier".into()),
                            found: other,
                        }
                        .boxed());
                    }
                },
                Subexpression { subexpression } => match token {
                    Token::Punctuation(PunctuationToken::Dot) => {
                        self.state = StructMember { subexpression };
                    }
                    Token::Punctuation(PunctuationToken::SquareBrackets(
                        ParenthesisType::Opening,
                    )) => {
                        let inner = ExpressionParser::take_until_closing(
                            &mut tokens,
                            Token::Punctuation(PunctuationToken::SquareBrackets(
                                ParenthesisType::Closing,
                            )),
                        )?;
                        let index_expression = ExpressionParser::parse(inner, environment)?;

                        self.state = Subexpression {
                            subexpression: Expression::ArrayIndex(ArrayIndexExpression::new(
                                Box::new(subexpression),
                                Box::new(index_expression),
                            )),
                        };
                    }
                    Token::Punctuation(PunctuationToken::ThinArrow) => {
                        self.state = AssociatedProcedureCall {
                            subexpression,
                            ident: None,
                        }
                    }
                    other => {
                        return Err(CompilerError::UnexpectedToken {
                            expected: Some(
                                "Scope Address, Module Member or Associated Procedure Call".into(),
                            ),
                            found: other,
                        }
                        .boxed());
                    }
                },
                ModuleMember {
                    module_ident,
                    member_ident,
                } => {
                    if let Some(member_ident) = member_ident {
                        match token {
                            Token::Punctuation(PunctuationToken::Parenthesis(
                                ParenthesisType::Opening,
                            )) => {
                                let inner = ExpressionParser::take_until_closing(
                                    &mut tokens,
                                    Token::Punctuation(PunctuationToken::Parenthesis(
                                        ParenthesisType::Closing,
                                    )),
                                )?;
                                let raw_args = ExpressionParser::split_by_commas(inner)?;
                                let mut arguments = Vec::new();
                                for arg in raw_args {
                                    arguments.push(ExpressionParser::parse(arg, environment)?);
                                }

                                self.state = Subexpression {
                                    subexpression: Expression::ProcedureCall(ProcedureCallExpression::new(
                                        ModuleAddress::new(
                                            module_ident,
                                            member_ident,
                                        ),
                                        arguments,
                                    )),
                                };
                            }
                            Token::Punctuation(PunctuationToken::CurlyBraces(
                                ParenthesisType::Opening,
                            )) => {
                                let inner = ExpressionParser::take_until_closing(
                                    &mut tokens,
                                    Token::Punctuation(PunctuationToken::CurlyBraces(
                                        ParenthesisType::Closing,
                                    )),
                                )?;
                                let raw_field_overrides = ExpressionParser::split_by_commas(inner)?;
                                let mut field_overrides = Vec::new();
                                for field_override in raw_field_overrides {
                                    let mut tokens = field_override.into_iter();

                                    let field_name = tokens.next();
                                    let field_name = match field_name {
                                        Some(Token::Identifier(ident)) => ident,
                                        other => {
                                            return Err(CompilerError::UnexpectedToken {
                                                expected: Some("Identifier".into()),
                                                found: other
                                                    .unwrap_or(Token::Identifier("".into())),
                                            }
                                            .boxed());
                                        }
                                    };

                                    match tokens.next() {
                                        Some(Token::Punctuation(PunctuationToken::Colon)) => {}
                                        other => {
                                            return Err(CompilerError::UnexpectedToken {
                                                expected: Some(":".into()),
                                                found: other
                                                    .unwrap_or(Token::Identifier("".into())),
                                            }
                                            .boxed());
                                        }
                                    };

                                    field_overrides.push((
                                        field_name,
                                        ExpressionParser::parse(tokens, environment)?,
                                    ));
                                }

                                self.state = Subexpression {
                                    subexpression: Expression::StructConstruction(StructConstructionExpression::new(
                                        ModuleAddress::new(module_ident, member_ident),
                                        field_overrides,
                                    )),
                                };
                            }

                            other => {
                                return Err(CompilerError::UnexpectedToken {
                                    expected: Some("Procedure Call or Struct Construction".into()),
                                    found: other,
                                }
                                .boxed());
                            }
                        }
                    } else {
                        if let Token::Identifier(member_ident) = token {
                            self.state = ModuleMember {
                                module_ident,
                                member_ident: Some(member_ident),
                            }
                        } else {
                            return Err(CompilerError::UnexpectedToken {
                                expected: Some("Identifier".into()),
                                found: token,
                            }
                            .boxed());
                        }
                    }
                }
                StructMember { subexpression } => match token {
                    Token::Identifier(member_ident) => {
                        self.state = Subexpression {
                            subexpression: Expression::StructMember(StructMemberExpression::new(
                                Box::new(subexpression),
                                member_ident,
                            )),
                        }
                    }

                    other => {
                        return Err(CompilerError::UnexpectedToken {
                            expected: Some("Identifier".into()),
                            found: other,
                        }
                        .boxed());
                    }
                },
                AssociatedProcedureCall {
                    subexpression,
                    ident,
                } => {
                    if let Some(ident) = ident {
                        match token {
                            Token::Punctuation(PunctuationToken::Parenthesis(
                                ParenthesisType::Opening,
                            )) => {
                                let inner = ExpressionParser::take_until_closing(
                                    &mut tokens,
                                    Token::Punctuation(PunctuationToken::Parenthesis(
                                        ParenthesisType::Closing,
                                    )),
                                )?;
                                let raw_args = ExpressionParser::split_by_commas(inner)?;
                                let mut arguments = Vec::new();
                                for arg in raw_args {
                                    arguments.push(ExpressionParser::parse(arg, environment)?);
                                }

                                self.state = Subexpression {
                                    subexpression: Expression::AssociatedProcedureCall(AssociatedProcedureCallExpression::new(
                                        Box::new(subexpression),
                                        ident,
                                        arguments,
                                    )),
                                };
                            }

                            other => {
                                return Err(CompilerError::UnexpectedToken {
                                    expected: Some("(".into()),
                                    found: other,
                                }
                                .boxed());
                            }
                        }
                    } else {
                        match token {
                            Token::Identifier(ident) => {
                                self.state = AssociatedProcedureCall {
                                    subexpression,
                                    ident: Some(ident),
                                };
                            }

                            other => {
                                return Err(CompilerError::UnexpectedToken {
                                    expected: Some("Identifier".into()),
                                    found: other,
                                }
                                .boxed());
                            }
                        }
                    }
                }
            }
        }

        match self.state {
            ExpressionAtomParserState::Base => Err(CompilerError::InvalidExpression {
                message: "Empty subexpression atom!".into(),
            }
            .boxed()),
            ExpressionAtomParserState::SingleIdent { ident } => Ok(ExpressionAtom::Subexpression(
                Expression::Variable(VariableExpression::new(
                    vec![VariableAddressant::Identifier(ident)].try_into().unwrap(),
                    VariableAccessMode::Move
                )),
            )),
            ExpressionAtomParserState::ScopeAddress { address, access } => {
                Ok(ExpressionAtom::Subexpression({
                    Expression::Variable(VariableExpression::new(
                        address.try_into().unwrap(), access
                    ))
                }))
            }
            ExpressionAtomParserState::ScopeAddressMember {
                address: _,
                access: _,
            } => Err(CompilerError::InvalidExpression {
                message: "Missing token. Expected identifier after '.'!".into(),
            }
            .boxed()),
            ExpressionAtomParserState::Subexpression { subexpression } => {
                Ok(ExpressionAtom::Subexpression(subexpression))
            }
            ExpressionAtomParserState::ModuleMember {
                module_ident,
                member_ident,
            } => {
                if let Some(member_ident) = member_ident {
                    Ok(ExpressionAtom::Subexpression(Expression::Value(Value::Type(
                        Type::Struct {
                            struct_id: ModuleAddress::new(module_ident, member_ident),
                        },
                    ))))
                } else {
                    Err(CompilerError::InvalidExpression {
                        message: "Incomplete subexpression!".into(),
                    }
                    .boxed())
                }
            }

            ExpressionAtomParserState::StructMember { subexpression: _ } => {
                Err(CompilerError::InvalidExpression {
                    message: "Missing token. Expected identifier after '.'!".into(),
                }
                .boxed())
            }
            ExpressionAtomParserState::AssociatedProcedureCall {
                subexpression: _,
                ident: _,
            } => Err(CompilerError::InvalidExpression {
                message: "Incomplete associated procedure call!".into(),
            }
            .boxed()),
        }
    }
}

fn try_parse_literal(literal: crate::lexer::token::LiteralToken) -> Result<Value> {
    match literal {
        LiteralToken::Null => Ok(Value::Null),
        LiteralToken::Integer(num) => Ok(Value::Integer(num.parse().map_err(|_| {
            CompilerError::LiteralParseError {
                ty: Type::Integer,
                literal: num,
            }
            .boxed()
        })?)),
        LiteralToken::Float(num) => Ok(Value::Float(num.parse().map_err(|_| {
            CompilerError::LiteralParseError {
                ty: Type::Float,
                literal: num,
            }
            .boxed()
        })?)),
        LiteralToken::Boolean(b) => match &b as &str {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(CompilerError::LiteralParseError {
                ty: Type::Bool,
                literal: b,
            }
            .boxed()),
        },
        LiteralToken::Char(c) => Ok(Value::Char(
            c.chars().next().ok_or(
                CompilerError::LiteralParseError {
                    ty: Type::Char,
                    literal: c,
                }
                .boxed(),
            )?,
        )),
        LiteralToken::String(str) => Ok(Value::String(str)),
        LiteralToken::Type(ty) => Ok(Value::Type(parse_type(ty))),
    }
}

fn parse_type(ty: PrimitiveTypeToken) -> Type {
    macro_rules! id {
        ($value:ident: $id0:ident $(, $id:ident)+) => {
            match $value {
                PrimitiveTypeToken::$id0 => Type::$id0,
                $(
                    PrimitiveTypeToken::$id => Type::$id,
                )+
            }
        };
    }
    
    id!(ty: Null, Integer, Float, Bool, Char, String, Array, Moved, Dropped, Type)   
}