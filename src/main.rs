use std::env;

use otr::{compiler::{Compiler, file_reader::{FileReader, ImportAddress}}, lexer::Tokenizer, runtime::RuntimeObject};

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

    let compiled_object = match compiler.compile() {
        Ok(obj) => obj,
        Err(error) => {
            println!("{error}");
            return;
        }
    };

    println!("{:#?}", compiled_object);
            
            

    let runtime_object = RuntimeObject::from(compiled_object);

    if let Err(error) = runtime_object.execute() {
        println!("{error}");
    }
}
