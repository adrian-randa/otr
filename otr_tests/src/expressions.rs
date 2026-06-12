use otr_compiler::{ExpressionParser, Fragmenter, LiteralToken, NoExpressionEnvironment, PrimitiveTypeToken, Token, TokenStream, Tokenizer};
use otr_core::{expression::Expression, r#type::Type, value::Value};
use otr_runtime::EnvironmentBuilder;




#[test]
fn basic_values() {
    let instances = [
        (TokenStream(vec![Token::Literal(LiteralToken::Integer("5".into()))]), Value::Integer(5)),
        (TokenStream(vec![Token::Literal(LiteralToken::Integer("-3".into()))]), Value::Integer(-3)),
        (TokenStream(vec![Token::Literal(LiteralToken::Integer("-0".into()))]), Value::Integer(0)),
        (TokenStream(vec![Token::Literal(LiteralToken::Float("10.0".into()))]), Value::Float(10.0)),
        (TokenStream(vec![Token::Literal(LiteralToken::Float("-14.637".into()))]), Value::Float(-14.637)),
        (TokenStream(vec![Token::Literal(LiteralToken::Boolean("true".into()))]), Value::Bool(true)),
        (TokenStream(vec![Token::Literal(LiteralToken::Boolean("false".into()))]), Value::Bool(false)),
        (TokenStream(vec![Token::Literal(LiteralToken::Char("f".into()))]), Value::Char('f')),
        (TokenStream(vec![Token::Literal(LiteralToken::Char("\n".into()))]), Value::Char('\n')),
        (TokenStream(vec![Token::Literal(LiteralToken::Char("5".into()))]), Value::Char('5')),
        (TokenStream(vec![Token::Literal(LiteralToken::String("\"abcd ef ghij \n\n klm\"".into()))]), Value::String("\"abcd ef ghij \n\n klm\"".into())),
        (TokenStream(vec![Token::Literal(LiteralToken::String("".into()))]), Value::String("".into())),
        (TokenStream(vec![Token::Literal(LiteralToken::Type(PrimitiveTypeToken::Null))]), Value::Type(Type::Null)),
    ];

    let environment = NoExpressionEnvironment;

    for (tokens, expected) in instances {
        let tokens_str = format!("{:?}", tokens);
        let expression = ExpressionParser::parse(tokens, &environment).unwrap();

        assert_eq!(expression, Expression::Value(expected), "testing {}", tokens_str);
    }
}

#[test]
fn basic_arithmetic_strings() {
    let instances = [
        ("-1 + 0", Value::Integer(-1)),
        ("50 - 51", Value::Integer(-1)),
        ("0 + 10 - 2", Value::Integer(8)),
        ("2.0 * 2.0", Value::Float(4.0)),
        ("6 / -2", Value::Integer(-3)),
        ("5 + 5 * 2", Value::Integer(15)),
        ("5 * 5 + 2", Value::Integer(27)),
        ("((5) * (5 + 2))", Value::Integer(35)),
        ("10 % 8", Value::Integer(2)),
        ("-1 % 8", Value::Integer(7)),

        ("20 / 2 / 2", Value::Integer(5)),
        ("20 / (2 / 2)", Value::Integer(20)),
        ("20 - 2 - 2", Value::Integer(16)),
        ("20 - (2 - 2)", Value::Integer(20)),
        
        ("2 ^ 3 ^ 2", Value::Integer(512)),
        ("(2 ^ 3) ^ 2", Value::Integer(64)),
    ];

    let environment = EnvironmentBuilder::new().build();

    for (string, expected) in instances {
        let fragments = Fragmenter::fragment(string).unwrap();
        let tokens = Tokenizer::default().tokenize(fragments).unwrap();
        let expression = ExpressionParser::parse(tokens.into_iter().map(|t| t.token), &NoExpressionEnvironment).unwrap();

        let result= otr_runtime::expressions::eval_expression(&expression, &environment).unwrap();

        assert_eq!(result, expected, "testing {}", string);
    }
}


#[test]
fn boolean_strings() {
    let instances = [
        ("true || false", Value::Bool(true)),
        ("false || true", Value::Bool(true)),
        ("false || false", Value::Bool(false)),

        
        ("true && true || false", Value::Bool(true)),
        ("true || true && false", Value::Bool(true)),
        ("(true || true) && false", Value::Bool(false)),

        ("!true", Value::Bool(false)),
        ("!true || true && false", Value::Bool(false)),
        ("true || !false && false", Value::Bool(true)),
    ];

    let environment = EnvironmentBuilder::new().build();

    for (string, expected) in instances {
        let fragments = Fragmenter::fragment(string).unwrap();
        let tokens = Tokenizer::default().tokenize(fragments).unwrap();
        let expression = ExpressionParser::parse(tokens.into_iter().map(|t| t.token), &NoExpressionEnvironment).unwrap();

        let result= otr_runtime::expressions::eval_expression(&expression, &environment).unwrap();



        assert_eq!(result, expected, "testing {}", string);
    }
}

#[test]
fn string_concatenation_strings() {
    let instances = [
        ("\"Hello\" + \" \" + \"World!\"", Value::String("Hello World!".into())),
        ("\"a\" + 2", Value::String("a2".into())),
        ("\"abc\" + 4 + \"def\" + 2 + 1", Value::String("abc4def21".into())),
        ("\"\" + \"\"", Value::String("".into())),
    ];

    let environment = EnvironmentBuilder::new().build();

    for (string, expected) in instances {
        let fragments = Fragmenter::fragment(string).unwrap();
        let tokens = Tokenizer::default().tokenize(fragments).unwrap();
        let expression = ExpressionParser::parse(tokens.into_iter().map(|t| t.token), &NoExpressionEnvironment).unwrap();

        let result= otr_runtime::expressions::eval_expression(&expression, &environment).unwrap();



        assert_eq!(result, expected, "testing {}", string);
    }
}

#[test]
fn equality_and_ordering_strings() {
    let instances = [
        ("2 + 3 == 5", Value::Bool(true)),
        ("2 + 3 == 5 - 1", Value::Bool(false)),

        ("1 == 2 == false", Value::Bool(true)),
        ("1 == (2 == false)", Value::Bool(false)),

        ("1 != 2", Value::Bool(true)),
        ("1 < 2", Value::Bool(true)),
        ("9001 > 9000", Value::Bool(true)),
        ("42 >= 42", Value::Bool(true)),
        ("42 > 42", Value::Bool(false)),
        ("42 <= 42", Value::Bool(true)),
        ("42 < 42", Value::Bool(false)),
    ];

    let environment = EnvironmentBuilder::new().build();

    for (string, expected) in instances {
        let fragments = Fragmenter::fragment(string).unwrap();
        let tokens = Tokenizer::default().tokenize(fragments).unwrap();
        let expression = ExpressionParser::parse(tokens.into_iter().map(|t| t.token), &NoExpressionEnvironment).unwrap();

        let result= otr_runtime::expressions::eval_expression(&expression, &environment).unwrap();



        assert_eq!(result, expected, "testing {}", string);
    }
}