use std::{cell::RefCell, collections::HashMap, env, fs::{self, read_to_string}, rc::Rc, str::FromStr};

use otr::{compiler::{Compiler, expression_parser::ExpressionParser, file_reader::{FileReader, ImportAddress}}, lexer::{FragmentStream, Tokenizer, token::{PunctuationToken, Token}}, runtime::{
    Expression, ModuleAddress, Scope, ScopeAddressant, Struct, Value, environment::Environment, expressions::{
        EqualityExpression, ProcedureCallExpression, VariableExpression, arithmetic::AddExpression, boolean::NotExpression
    }, module::Module, procedures::{CompiledProcedure, CompiledProcedureBuilder, Instruction, Procedure}
}};

fn main() {
    
    /* let input = "Dere::Saft { saftigkeit: 20 }";

    let fragments = FragmentStream::from_str(input).unwrap();

    let tokens = Tokenizer::default().tokenize(fragments).unwrap();

    println!("{:?}", ExpressionParser::parse(tokens)); */

    let mut file_reader = FileReader::new(env::current_dir().unwrap());

    let mut args = env::args();
    args.next();

    let module_name = args.next().unwrap();

    let main_module = ImportAddress {
        module_id: module_name,
        path: None,
    };

    //println!("Basepath {:?} | Module name {}", env::current_dir().unwrap(), module_name);

    file_reader.enqueue(main_module);

    let compiler = Compiler::new(file_reader);

    let runtime_object = compiler.compile().unwrap();
    
    println!("{:?}", runtime_object.execute());
}