use std::{env, path::PathBuf};

use clap::{self, Arg, Command, Parser, builder::ValueParser};
use otr_core::SystemError;

fn main() {
    if let Err(err) = shell() {
        println!("{}", err);
    }
}

fn shell() -> otr_core::Result<()> {
    let cli = CLI::parse();

    let root_module_id = cli.root_module_id;
    let root_path = cli.root_path.unwrap_or(
        env::current_dir()
            .map_err(|io_err| {
                SystemError::new(format!("Could not get current directory: {}", io_err)).boxed()
            })?
    );

    otrc::compile_and_write_dependency_tree(&root_path, root_module_id)?;


    Ok(())
}

#[derive(Parser, Debug)]
#[command(about = "A CLI for compiling OTR projects and scripts")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct CLI {
    pub root_module_id: String,

    #[arg(
        short = 'P',
        long = "path",
        help = "The path to the project's directory"
    )]
    pub root_path: Option<PathBuf>,
}
