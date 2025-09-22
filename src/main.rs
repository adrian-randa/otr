use std::{cell::RefCell, collections::HashMap, fs::read_to_string, rc::Rc};

use otr::runtime::{expressions::{arithmetic::AddExpression, Expression, ProcedureCallExpression, VariableExpression}, procedures::{CompiledProcedure, Instruction, Procedure}, Environment, Object, Scope, ScopeAddressant, Value};

fn main() {
    
    let mut environment = Environment::new();

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

    let expr = ProcedureCallExpression {
        procedure_id: "test_proc".into(),
        arguments: vec![],
    };

    {
        let procedures = Rc::get_mut(&mut environment.procedures).unwrap();
        procedures.insert("test_proc".into(), Box::new(test_proc));
    }

    let returned = expr.eval(&environment);

    println!("{:?}", returned);
}
