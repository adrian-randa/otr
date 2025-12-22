use std::{cell::RefCell, collections::HashMap, fs::{self, read_to_string}, rc::Rc, str::FromStr};

use otr::{compiler::{Compiler, expression_parser::ExpressionParser}, lexer::{FragmentStream, Tokenizer, token::{PunctuationToken, Token}}, runtime::{
    Expression, ModuleAddress, Scope, ScopeAddressant, Struct, Value, environment::Environment, expressions::{
        EqualityExpression, ProcedureCallExpression, VariableExpression, arithmetic::AddExpression, boolean::NotExpression
    }, module::Module, procedures::{CompiledProcedure, CompiledProcedureBuilder, Instruction, Procedure, builtin::SizeProcedure}
}};

fn main() {
    
    let input = "
    module Dere { 
        @entrypoint
        proc saftlhuaba() {
            let a = 5;
            return Math::double(a + 1);
        }

        export saftlhuaba;
    }
    
    module Math {

        proc double(x) {
            return x * 2;
        }

        export double;
    }
    
    ";

    let fragments = FragmentStream::from_str(input).unwrap();

    let tokens = Tokenizer::default().tokenize(fragments).unwrap();

    let mut compiler = Compiler::new();

    for token in tokens {
        compiler = compiler.read(token).unwrap();
    }

    let runtime_object = compiler.finalize().unwrap();

    println!("{:?}", runtime_object.execute());
}