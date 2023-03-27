use std::time::Duration;

use clap::{Parser, Subcommand};

mod cargo;
mod crates;
mod helper;
mod local_crates;
mod package_installer;
mod package_updater;

use crates::{CratesIoClient, CratesRegistry};
use crates_index::DependencyKind;
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
        #[clap(short, long, action, help = "Use cached crates index")]
        cached: bool,
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
    #[clap(name = "info", about = "Show package info")]
    Info {
        #[clap(name = "package", action, help = "Package name")]
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
    let mut registry = CratesRegistry::new();

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
            cached,
        } => {
            if !cached {
                let progress_bar = indicatif::ProgressBar::new_spinner();
                progress_bar.set_message("Updating crates index...");
                progress_bar.enable_steady_tick(Duration::from_millis(100));
                registry._update_index()?;
                progress_bar.finish_and_clear();
            }

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

        Commands::Info { package } => {
            let mut client = CratesIoClient::new();
            let progress_bar = indicatif::ProgressBar::new_spinner();
            progress_bar.set_message("Fetching package info...");
            progress_bar.enable_steady_tick(Duration::from_millis(100));
            let package_1 = registry.get_crate(&package)?;
            let package_2 = client.get_package_info(package)?;
            progress_bar.finish_and_clear();

            let Some(highest_version) = package_1.highest_normal_version() else {
                println!("Unable to find any suitable version for package {}.", package_1.name().blue());
                return Ok(())
            };

            // Print package name and version
            println!(
                "{} {}",
                highest_version.name(),
                highest_version.version().bright_black()
            );

            // Print description
            if let Some(description) = package_2.description {
                if let Ok((terminal_width, _)) = termion::terminal_size() {
                    let description =
                        textwrap::fill(&description, terminal_width.saturating_sub(4) as usize);
                    println!("  Description");
                    println!(
                        "    {}",
                        description
                            .lines()
                            .collect::<Vec<_>>()
                            .join("\n    ")
                            .bright_black()
                    );
                }
            }

            let yanked_version_count = package_1
                .versions()
                .iter()
                .filter(|version| version.is_yanked())
                .count();

            // Print version count
            println!(
                "  {} published versions ({} yanked)",
                package_1.versions().len().cyan(),
                yanked_version_count.cyan()
            );

            // Print dependency count
            println!(
                "  {} {}",
                highest_version.dependencies().len().cyan(),
                helper::pluralize(
                    "dependency",
                    "dependencies",
                    highest_version.dependencies().len()
                )
            );

            // Sort dependencies by name and kind
            let sorted_dependencies = {
                let mut deps = highest_version.dependencies().to_vec();
                deps.sort_unstable_by_key(|dep| dep.crate_name().to_string());
                deps.sort_by(|a, b| {
                    use std::cmp::Ordering;
                    match (a.kind(), b.kind()) {
                        (a, b) if a == b => Ordering::Equal,
                        (DependencyKind::Normal, _) => Ordering::Less,
                        (DependencyKind::Dev, DependencyKind::Build) => Ordering::Less,
                        _ => Ordering::Greater,
                    }
                });
                deps
            };

            // Print dependencies
            for dependency in sorted_dependencies {
                println!(
                    "    {kind}{crate_name} {version_req}",
                    kind = match dependency.kind() {
                        DependencyKind::Normal => "",
                        DependencyKind::Dev => "dev ",
                        DependencyKind::Build => "build ",
                    }
                    .magenta(),
                    crate_name = dependency.crate_name(),
                    version_req = dependency.requirement().bright_black(),
                );
            }

            // Print feature count
            println!(
                "  {} {}",
                highest_version.features().len().cyan(),
                helper::pluralize("feature", "features", highest_version.features().len())
            );

            // Sort features by name
            let sorted_features = {
                let mut feats = highest_version.features().iter().collect::<Vec<_>>();
                feats.sort_by_key(|(name, _)| name.to_string());
                feats
            };

            // Print features
            for (feature_name, sub_features) in sorted_features {
                println!(
                    "    {name} = [{flags}]",
                    name = feature_name,
                    flags = sub_features.join(", ").bright_black(),
                );
            }
        }
    }

    Ok(())
}
