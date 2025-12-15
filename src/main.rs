use std::{cell::RefCell, collections::HashMap, fs::{self, read_to_string}, rc::Rc, str::FromStr};

use otr::{compiler::expression_parser::ExpressionParser, lexer::{FragmentStream, Tokenizer, token::{PunctuationToken, Token}}, runtime::{
    Expression, ModuleAddress, Scope, ScopeAddressant, Struct, Value, environment::Environment, expressions::{
        EqualityExpression, ProcedureCallExpression, VariableExpression, arithmetic::AddExpression, boolean::NotExpression
    }, module::Module, procedures::{CompiledProcedure, Instruction, Procedure, builtin::SizeProcedure}
}};

fn main() {
    
    let input = "5 + 30 * test.boo.baz[i * 3 + 1] / (5 + z)";

    let fragments = FragmentStream::from_str(input).unwrap();

    let tokens = Tokenizer::default().tokenize(fragments).unwrap();

    let expression = ExpressionParser::parse(tokens).unwrap();

    println!("{:?}", expression);
}



fn runtime_test() {
    let mut builtin_mod = Module::default();

    builtin_mod.insert_procedure("size".into(), Box::new(SizeProcedure), true);

    let mut environment = Environment::new("not_main".into());

    let test_proc = CompiledProcedure {
        arguments_identifiers: vec![],
        instructions: vec![
            Instruction::PushVarToScope {
                identifier: "a".into(),
            },
            Instruction::EvaluateExpression {
                expression: Box::new(Value::Array(vec![
                    Value::Null,
                    Value::Null,
                    Value::Null,
                    Value::Null,
                    Value::Null,
                ])),
                target: Some(
                    vec![ScopeAddressant::Identifier("a".into())]
                        .try_into()
                        .unwrap(),
                ),
            },
            Instruction::PushVarToScope {
                identifier: "l".into(),
            },
            Instruction::EvaluateExpression {
                expression: Box::new(ProcedureCallExpression {
                    procedure_id: ("builtin", "size").into(),
                    arguments: vec![Box::new(VariableExpression {
                        variable_address: vec!["a".into()].try_into().unwrap(),
                    })],
                }),
                target: Some(vec!["l".into()].try_into().unwrap()),
            },
            Instruction::PushVarToScope {
                identifier: "i".into(),
            },
            Instruction::EvaluateExpression {
                expression: Box::new(Value::Integer(0)),
                target: Some(vec!["i".into()].try_into().unwrap()),
            },
            Instruction::EvaluateExpression {
                expression: Box::new(VariableExpression {
                    variable_address: vec!["i".into()].try_into().unwrap(),
                }),
                target: Some(
                    vec![
                        ScopeAddressant::Identifier("a".into()),
                        ScopeAddressant::DynamicIndex(Rc::new(VariableExpression {
                            variable_address: vec!["i".into()].try_into().unwrap(),
                        })),
                    ]
                    .try_into()
                    .unwrap(),
                ),
            },
            Instruction::EvaluateExpression {
                expression: Box::new(AddExpression::new(
                    Box::new(VariableExpression {
                        variable_address: vec!["i".into()].try_into().unwrap(),
                    }),
                    Box::new(Value::Integer(1)),
                )),
                target: Some(vec!["i".into()].try_into().unwrap()),
            },
            Instruction::JumpConditional {
                condition_expression: Box::new(NotExpression::new(Box::new(
                    EqualityExpression::new(
                        Box::new(VariableExpression {
                            variable_address: vec!["l".into()].try_into().unwrap(),
                        }),
                        Box::new(VariableExpression {
                            variable_address: vec!["i".into()].try_into().unwrap(),
                        }),
                    ),
                ))),
                jump_target: 6,
            },
            Instruction::PushVarToScope {
                identifier: "foo".into(),
            },
            Instruction::EvaluateExpression {
                expression: Box::new(VariableExpression {
                    variable_address: vec![ScopeAddressant::Identifier("a".into())]
                        .try_into()
                        .unwrap(),
                }),
                target: Some(
                    vec![ScopeAddressant::Identifier("foo".into())]
                        .try_into()
                        .unwrap(),
                ),
            },
            Instruction::Return {
                expression: Box::new(Value::String("Hii :3".into())),
            },
        ],
    };

    let mut module = Module::default();

    module.insert_procedure("test_proc".into(), Box::new(test_proc), true);

    let expr = ProcedureCallExpression {
        procedure_id: ("main", "test_proc").into(),
        arguments: vec![],
    };

    environment.load_module("main".into(), Rc::new(module));
    environment.load_module("builtin".into(), Rc::new(builtin_mod));

    let returned = expr.eval(&environment);

    println!("{:?}", returned);


    let input = fs::read_to_string("./test_inputs/test01.otr").unwrap();

    let frags = FragmentStream::from_str(&input).unwrap();

    println!("{:?}", Tokenizer::default().tokenize(frags));
}
