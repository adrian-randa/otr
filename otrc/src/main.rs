use std::{env, path::PathBuf, process::exit};

use clap::Parser;
use otr_config::get_global_config;
use otr_core::SystemError;

fn main() {
    if let Err(err) = shell() {
        println!("{}", err);
        exit(1);
    }
}

fn shell() -> otr_core::Result<()> {
    let cli = Cli::parse();

    let root_module_id = cli.root_module_id;
    let root_path = cli.root_path.unwrap_or(
        env::current_dir()
            .map_err(|io_err| {
                SystemError::new(format!("Could not get current directory: {}", io_err)).boxed()
            })?
    );

    let global_configuration = match cli.global_configuration_path {
        Some(config_path) => Some(get_global_config(&config_path)?),
        None => None,
    };

    otrc::compile_and_write_dependency_tree(&root_path, root_module_id, global_configuration)?;


    Ok(())
}

#[derive(Parser, Debug)]
#[command(about = "A CLI for compiling OTR projects and scripts")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    pub root_module_id: String,

    #[arg(
        short = 'P',
        long = "path",
        help = "The path to the project's directory"
    )]
    pub root_path: Option<PathBuf>,

    #[arg(
        short = 'G',
        long = "globals",
        help = "The path to the global config file"
    )]
    pub global_configuration_path: Option<PathBuf>
}