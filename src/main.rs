use std::{cell::RefCell, collections::HashMap, fs::read_to_string, rc::Rc};

use otr::runtime::{expressions::{AddExpression, Expression, ProcedureCallExpression}, procedures::{CompiledProcedure, Instruction, Procedure}, Environment, Object, Scope, ScopeAddressant, Value};

fn main() {
    
    let mut environment = Environment::new();

    let test_proc = CompiledProcedure {
        arguments_identifiers: vec![],
        instructions: vec![
            Instruction::PushVarToScope { identifier: "a".into() },
            Instruction::EvaluateExpression {
                expression: Box::new(Value::Integer(2)),
                target: Some(vec![ScopeAddressant::Identifier("a".into())].into())
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
