use std::time::Instant;

use clap::{Parser, Subcommand};
use collar::{cmd::compile_project, compile_and_run_project, error::CollarError, fs::{ensure_dir_exists, get_current_dir, get_installed_versions, install_otrc, install_otrrun, uninstall_otrc, uninstall_otrrun}, get_config, net::{download_otrc, download_otrrun}, new_executable, new_library};
use colored::Colorize;
use otr_config::Version;

use otr_core::Result;

fn main() {
    if let Err(err) = shell() {
        println!("{err}");
    }
}

fn shell() -> Result<()> {
    let cli = CLI::parse();

    let current_dir = get_current_dir();

    match cli {
        CLI::New(new_command) => {
            match new_command {
                NewCommand::Executable { name, version: version_str} => {
                    let version;
                    if let Some(v) = version_str {
                        version =
                        Version::try_from(&v as &str)?;
                    } else {
                        version = get_installed_versions()?
                            .latest_otrc()
                            .ok_or(
                                CollarError::new("Could not find latest installed version").boxed()
                            )?;
                    }
                    let project_path = current_dir?.join(&name);
                    ensure_dir_exists(&project_path)?;
                    new_executable(&project_path, name, version)?;

                    success(format!("Created project at {}", project_path.display()))
                },
                NewCommand::Library { name , version: version_str} => {
                    let version;
                    if let Some(v) = version_str {
                        version =
                        Version::try_from(&v as &str)?;
                    } else {
                        version = get_installed_versions()?
                            .latest_otrc()
                            .ok_or(
                                CollarError::new("Could not find latest installed version").boxed()
                            )?;
                    }

                    let project_path = current_dir?.join(&name);
                    ensure_dir_exists(&project_path)?;
                    new_library(&project_path, name, version)?;

                    success(format!("Created project at {}", project_path.display()))
                },
            }
        },
        CLI::List => {
            let versions = get_installed_versions()?;

            println!("{}", "Installed 'otrc' versions:".blue());
            if versions.otrc.is_empty() {
                println!("    {}", "none".red());
            } else {
                for version in versions.otrc {
                    println!("    {version}")
                }
            }

            println!("{}", "Installed 'otrrun' versions:".blue());
            if versions.otrrun.is_empty() {
                println!("    {}", "none".red());
            } else {
                for version in versions.otrrun{
                    println!("    {version}")
                }
            }

            Ok(())
        },
        CLI::Run => {
            let current_dir = current_dir?;

            let config = get_config(&current_dir)?;

            compile_and_run_project(&current_dir, &config.root_module)
        },
        CLI::Compile => {
            let current_dir = current_dir?;

            let config = get_config(&current_dir)?;
            
            let now = Instant::now();
            compile_project(&current_dir, &config.root_module)?;
            let elapsed = now.elapsed();

            success(format!("Compiled project in {:?}", elapsed))
        },
        CLI::Install(install_command) => {
            let version = match install_command.version {
                Some(v) => Some(Version::try_from(&v as &str)?),
                None => None,
            };
            
            match install_command.module {
                InstallKind::Full => {
                    let (otrc_version, otrc_bin) = download_otrc(version)?;
                    install_otrc(otrc_version, otrc_bin)?;

                    let (otrrun_version, otrrun_bin) = download_otrrun(version)?;
                    install_otrrun(otrrun_version, otrrun_bin)?;

                    success(format!("Installed otrc/{otrc_version} and otrrun/{otrrun_version}!"))
                },
                InstallKind::Otrc => {
                    let (otrc_version, otrc_bin) = download_otrc(version)?;
                    install_otrc(otrc_version, otrc_bin)?;

                    success(format!("Installed otrc/{otrc_version}!"))
                },
                InstallKind::Otrrun => {
                    let (otrrun_version, otrrun_bin) = download_otrrun(version)?;
                    install_otrrun(otrrun_version, otrrun_bin)?;

                    success(format!("Installed otrrun/{otrrun_version}!"))
                },
            }
        },
        CLI::Uninstall(uninstall_command) => {
            let version = Version::try_from(&uninstall_command.version as &str)?;

            match uninstall_command.module {
                UninstallKind::Full => uninstall_otrc(version).and(uninstall_otrrun(version)),
                UninstallKind::Otrc => uninstall_otrc(version),
                UninstallKind::Otrrun => uninstall_otrrun(version),
            }?;

            success("Uninstalled!")
        },
    }
}

fn success(message: impl AsRef<str>) -> Result<()> {
    println!("{} {}", " \u{2713} ".white().on_blue(), message.as_ref().blue());

    Ok(())
}


#[derive(Debug, Parser)]
#[command(about = "An OTR project and installation manager cli")]
enum CLI {
    #[command(about = "Create a new OTR project")]
    #[command(subcommand)]
    New(NewCommand),

    #[command(about = "List all installed modules")]    
    List,

    #[command(about = "Compile (if necessary) and run an OTR project")]
    Run,

    #[command(about = "Compile an OTR project")]
    Compile,

    #[command(about = "Install OTR module(s)")]
    Install(InstallCommand),

    #[command(about = "Remove OTR module(s)")]
    Uninstall(UninstallCommand),
}

#[derive(Debug, Subcommand)]
enum NewCommand {
    #[command(
        short_flag = 'E',
        long_flag = "exe",
        about = "Create an executable project"
    )]
    Executable {
        #[arg(help = "The name of the executable to create")]
        name: String,

        #[arg(help = "The version of otr to use for the new executable")]
        version: Option<String>
    },

    #[command(
        short_flag = 'L',
        long_flag = "lib",
        about = "Create a library project"
    )]
    Library {
        #[arg(help = "The name of the library to create")]
        name: String,

        #[arg(help = "The version of otr to use for the new library")]
        version: Option<String>
    },
}

#[derive(Debug, Parser)]
struct InstallCommand {
    #[command(subcommand)]
    module: InstallKind,

    #[arg(
        short = 'V',
        long = "version",
        help = "Specify the version to install",
    )]
    version: Option<String>,

    #[arg(
        short = 'R',
        long = "replace",
        help = "Force the installation, even if the specified version is already present",
        default_value = "false",
    )]
    force: bool,
}

#[derive(Debug, Subcommand, Clone)]
#[command(about = "Specify what module(s) to install")]
enum InstallKind {
    #[command(about = "Install both the compiler and runtime")]
    Full,
    
    #[command(about = "Install the compiler")]
    Otrc,

    #[command(about = "Install the runtime")]
    Otrrun
}

#[derive(Debug, Parser)]
struct UninstallCommand {
    #[command(subcommand)]
    module: UninstallKind,

    #[arg(help = "Specify the version to uninstall")]
    version: String,
}

#[derive(Debug, Subcommand, Clone)]
#[command(about = "Specify what module(s) to uninstall")]
enum UninstallKind {
    #[command(about = "Uninstall both the compiler and runtime")]
    Full,
    
    #[command(about = "Uninstall the compiler")]
    Otrc,

    #[command(about = "Uninstall the runtime")]
    Otrrun
}