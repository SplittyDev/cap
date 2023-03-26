use std::time::Duration;

use clap::{Parser, Subcommand};

mod cargo;
mod crates;
mod helper;
mod local_crates;
mod package_installer;
mod package_updater;

use crates::CratesRegistry;
use local_crates::{Package, PackageFormatting, PackageTree};
use owo_colors::OwoColorize;
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
    #[clap(name = "search", about = "Search for packages")]
    Search {
        #[clap(name = "package", action, help = "Package regex")]
        package: String,
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

        Commands::Search { package } => {
            let progress_bar = indicatif::ProgressBar::new_spinner();
            progress_bar.set_message("Searching for packages...");
            progress_bar.enable_steady_tick(Duration::from_millis(100));
            let packages = registry.search(package)?;
            let local_packages = PackageTree::build()?;
            progress_bar.finish_and_clear();
            for package in packages {
                let primary_text = format!(
                    "{} {}",
                    package.name().blue(),
                    package.version().bright_black()
                );
                let secondary_text = {
                    if let Some(local_package) = local_packages.get(package.name()) {
                        if local_package.version() == package.version() {
                            format!("({}, {})", "installed".cyan(), "up to date".green())
                        } else {
                            format!(
                                "({} {}: {})",
                                "installed".cyan(),
                                "out of date".yellow(),
                                package.version()
                            )
                        }
                    } else {
                        String::default()
                    }
                };
                let package_text = format!("{} {}", primary_text, secondary_text)
                    .trim()
                    .to_string();
                println!("{}", package_text);
            }
        }
    }

    Ok(())
}
