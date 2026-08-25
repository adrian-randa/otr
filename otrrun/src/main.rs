use std::{path::PathBuf, process::exit};

use clap::Parser;
use otr_core::{SystemError, module::ImportAddress};
use otrrun::{build_environment_from_features, build_runtime_object, get_current_dir, run_script};
use otr_config::{Version, get_global_config, get_project_config};

fn main() {
    if let Err(err) = shell() {
        println!("{}", err);
        exit(1);
    }
}

fn shell() -> otr_core::Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Script { object_path } => {
            run_script(&object_path)
        },
        Cli::Project { root_path, config_path, global_configuration_path } => {

            let root_path = root_path.unwrap_or(get_current_dir()?);

            let config_path = config_path.unwrap_or(
                get_current_dir()?.join("otr_config").with_extension("toml")
            );

            let config = get_project_config(&config_path)?;

            let config_version = config.get_otr_version();
            let own_version = Version::try_from(env!("CARGO_PKG_VERSION")).unwrap();

            if config_version != own_version {
                return Err(SystemError::new(
                    format!("Mismatched runtime versions! Project requires {config_version} but {own_version} was used.")
                ).boxed());
            }

            let root_module_address = ImportAddress {
                module_id: config.get_root_module().to_string(),
                path: None,
                alias: None,
            };

            let base_environment = build_environment_from_features(config.features())?;

            let global_configuration = if let Some(global_configuration_path) = global_configuration_path {
                Some(get_global_config(&global_configuration_path)?)
            } else {
                None
            };

            let runtime_object = build_runtime_object(&root_path, root_module_address, base_environment, global_configuration)?;

            runtime_object.execute()?;

            Ok(())
        },
    }
}

#[derive(Parser, Debug)]
#[command(about = "A CLI for compiling OTR projects and scripts")]
#[command(version = env!("CARGO_PKG_VERSION"))]
enum Cli {
    #[command(about = "Runs a single script file")]
    Script {
        #[arg(help = "The path to the project's directory")]
        object_path: PathBuf,
    },
    #[command(about = "Runs a project")]
    Project {
        #[arg(
            short = 'P',
            long = "path",
            help = "The path to the project's directory"
        )]
        root_path: Option<PathBuf>,

        #[arg(
            short = 'C',
            long = "config",
            help = "The path to the project's config file"
        )]
        config_path: Option<PathBuf>,

        #[arg(
            short = 'G',
            long = "globals",
            help = "The path to the global config file"
        )]
        global_configuration_path: Option<PathBuf>
    }
}