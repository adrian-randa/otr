use std::{
    cell::RefCell,
    collections::HashMap,
    env,
    fs::{self, read_to_string},
    rc::Rc,
    str::FromStr,
};

use otr::{
    compiler::{
        expression_parser::ExpressionParser,
        file_reader::{FileReader, ImportAddress},
        Compiler,
    },
    lexer::{
        fragmenter::FragmentStream,
        token::{PunctuationToken, Token},
        Tokenizer,
    },
    runtime::{
        environment::Environment,
        expressions::{
            arithmetic::AddExpression, boolean::NotExpression, EqualityExpression,
            ProcedureCallExpression, VariableExpression,
        },
        module::CompiledModule,
        procedures::{CompiledProcedure, CompiledProcedureBuilder, Instruction, Procedure},
        scope::{Scope, ScopeAddressant},
        Expression, ModuleAddress, Struct, Value,
    },
};

fn main() {
    let mut file_reader = FileReader::new(Tokenizer::default(), env::current_dir().unwrap());

    let mut args = env::args();
    args.next();

    let module_name = args.next().unwrap();

    let main_module = ImportAddress {
        module_id: module_name,
        path: None,
    };

    if let Err(error) = file_reader.push_dependency(main_module) {
        println!("{error}");
    }

    let compiler = Compiler::new(file_reader);

    let runtime_object = match compiler.compile() {
        Ok(obj) => obj,
        Err(error) => {
            println!("{error}");
            return;
        }
    };

    if let Err(error) = runtime_object.execute() {
        println!("{error}");
    }
}
