mod cargo;
mod crates;
mod helper;
mod local_crates;
mod package_installer;
mod package_updater;

use clap::{Parser, Subcommand};

use crates::CratesRegistry;
use local_crates::{Package, PackageFormatting, PackageTree};
use package_installer::PackageInstaller;
use package_updater::PackageUpdater;

#[derive(Debug, Subcommand)]
enum Commands {
    #[clap(name = "install", alias = "i", about = "Install a package")]
    Install {
        #[clap(name = "package", action, help = "The package to be installed")]
        package: String,
        #[clap(short = 'l', long = "locked", action, help = "Use crate lockfile")]
        locked: bool,
        #[clap(short = 'f', long = "forced", action, help = "Force installation")]
        forced: bool,
        #[clap(
            short = 'n',
            long = "nightly",
            action,
            help = "Use a nightly toolchain"
        )]
        nightly: bool,
    },
    #[clap(name = "update", about = "Update installed packages")]
    Update {
        #[clap(name = "package", action, help = "Update a specific package")]
        package: Option<String>,
    },
    #[clap(name = "check", about = "Check for updates")]
    Check {
        #[clap(name = "package", action, help = "Check a specific package")]
        package: Option<String>,
    },
    #[clap(name = "uninstall", about = "Remove a package")]
    Uninstall {
        #[clap(name = "package", action, help = "The package to be uninstalled")]
        package: String,
    },
    #[clap(name = "list", about = "List installed packages")]
    List {
        #[clap(short, long, action, help = "More compact output")]
        short: bool,
    },
}

#[derive(Debug, Parser)]
#[clap(name = "cap")]
#[command(author, version, about, long_about = None)]
struct App {
    #[clap(subcommand)]
    command: Commands,
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let registry = CratesRegistry::new();

    match app.command {
        Commands::Install {
            package,
            locked,
            forced,
            nightly,
        } => {
            let packages = PackageTree::build()?;
            let installer = PackageInstaller::new(&registry, &packages);
            installer.install_package(package, locked, forced, nightly)?;
        }

        Commands::Uninstall { package } => {
            let packages = PackageTree::build()?;
            let installer = PackageInstaller::new(&registry, &packages);
            installer.uninstall_package(package)?;
        }

        Commands::Check { package } => {
            let packages = PackageTree::build()?;
            let updater = PackageUpdater::new(&registry, &packages);

            if let Some(target_package) = package {
                updater.check_package(target_package)?;
            } else {
                updater.check_all_packages()?;
            }
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
