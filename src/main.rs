mod crates;
mod local_crates;
mod package_updater;

use clap::{Parser, Subcommand};

use crates::CratesRegistry;
use local_crates::{Package, PackageFormatting, PackageTree};
use package_updater::PackageUpdater;

#[derive(Debug, Subcommand)]
enum Commands {
    #[clap(name = "install", alias = "i", about = "Install a package")]
    Install {
        #[clap(long, action, help = "Use a nightly toolchain")]
        nightly: bool,
    },
    #[clap(name = "update", about = "Update installed packages")]
    Update {
        #[clap(name = "package", action, help = "Update a specific package")]
        package: Option<String>,
    },
    #[clap(name = "uninstall", about = "Remove a package")]
    Uninstall {},
    #[clap(name = "list", about = "List installed packages")]
    List {
        #[clap(short, long, action, help = "More compact output")]
        short: bool,
    },
}

#[derive(Debug, Parser)]
#[clap(name = "cap")]
struct App {
    #[clap(subcommand)]
    command: Commands,
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let registry = CratesRegistry::new();

    match app.command {
        Commands::Install { nightly: _nightly } => {
            todo!("Installing packages is not yet implemented.")
        }

        Commands::Uninstall {} => {
            todo!("Uninstalling packages is not yet implemented.")
        }

        Commands::Update {
            package: specific_package,
        } => {
            let packages = PackageTree::build()?;
            let updater = PackageUpdater::new(&registry, &packages);

            if let Some(target_package) = specific_package {
                updater.update_package(target_package)?;
            } else {
                updater.update_all_packages()?;
            }
        }

        Commands::List { short } => {
            let packages = PackageTree::build()?;
            let formatting = if short {
                PackageFormatting::Short
            } else {
                PackageFormatting::Long
            };
            packages.print(formatting);
        }
    }

    Ok(())
}
