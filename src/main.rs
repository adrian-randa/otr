use std::{cell::RefCell, collections::HashMap, fs::read_to_string, rc::Rc};

use otr::runtime::{expressions::{arithmetic::AddExpression, Expression, ProcedureCallExpression, VariableExpression}, procedures::{CompiledProcedure, Instruction, Procedure}, Environment, Module, Scope, ScopeAddressant, Struct, Value};

fn main() {
    
    let mut environment = Environment::new("not_main".into());

    let test_proc = CompiledProcedure {
        arguments_identifiers: vec![],
        instructions: vec![
            Instruction::PushVarToScope { identifier: "a".into() },
            Instruction::EvaluateExpression {
                expression: Box::new(Value::Array(vec![
                    Value::Integer(1),
                    Value::Integer(2),
                    Value::Integer(3),
                    Value::Integer(4),
                    Value::Integer(5),
                ])),
                target: Some(vec![ScopeAddressant::Identifier("a".into())].into())
            },
            Instruction::EvaluateExpression {
                expression: Box::new(Value::String("Hangula".into())),
                target: Some(vec![
                    ScopeAddressant::Identifier("a".into()),
                    ScopeAddressant::DynamicIndex(Rc::new(
                        Value::Integer(2)
                    )),
                ].into())
            },
            Instruction::PushVarToScope { identifier: "foo".into() },
            Instruction::EvaluateExpression {
                expression: Box::new(VariableExpression {
                    variable_address: vec![
                        ScopeAddressant::Identifier("a".into()),
                    ].into()
                }),
                target: Some(vec![ScopeAddressant::Identifier("foo".into())].into())
            },
            Instruction::Return { expression: Box::new(Value::String("Hii :3".into())) }
        ]
    };

    let mut module = Module::default();

    module.insert_procedure("test_proc".into(), Box::new(test_proc), false);

    let expr = ProcedureCallExpression {
        procedure_id: ("main", "test_proc").into(),
        arguments: vec![],
    };

    environment.load_module("main".into(), Rc::new(module));


    let returned = expr.eval(&environment);

    println!("{:?}", returned);
}
