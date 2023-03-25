use owo_colors::OwoColorize;

use crate::{CratesRegistry, Package, PackageTree};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PackageStatus {
    UpToDate,
    OutOfDate,
}

#[derive(Debug, Clone)]
pub struct PackageWithStatus<'a> {
    package: &'a Package,
    status: PackageStatus,
    latest_version: Option<semver::Version>,
}

impl<'a> PackageWithStatus<'a> {
    pub fn new(
        package: &'a Package,
        status: PackageStatus,
        latest_version: Option<semver::Version>,
    ) -> Self {
        Self {
            package,
            status,
            latest_version,
        }
    }

    pub fn package(&self) -> &'a Package {
        self.package
    }

    pub fn is_out_of_date(&self) -> bool {
        self.status == PackageStatus::OutOfDate
    }
}

pub struct PackageUpdater<'a> {
    registry: &'a CratesRegistry,
    packages: &'a PackageTree,
}

impl<'a> PackageUpdater<'a> {
    pub fn new(registry: &'a CratesRegistry, packages: &'a PackageTree) -> Self {
        Self { registry, packages }
    }

    pub fn check_package(
        &self,
        package_name: impl AsRef<str>,
    ) -> anyhow::Result<Option<PackageWithStatus>> {
        let Some(local_package) = self.packages.get(&package_name) else {
            println!("Package {} is {}.", package_name.as_ref().blue(), "not installed".red());
            return Ok(None);
        };

        let Ok(latest_version) = self.registry.get_latest_version(local_package.name()) else {
            println!("Package {} is {}.", local_package.name().blue(), "not available on crates.io".red());
            return Ok(None);
        };

        if latest_version <= *local_package.version() {
            println!(
                "Package {} is {}.",
                local_package.name().blue(),
                "up to date".green()
            );
            return Ok(Some(PackageWithStatus::new(
                local_package,
                PackageStatus::UpToDate,
                None,
            )));
        }

        println!(
            "Package {} is {} ({} -> {}).",
            local_package.name().blue(),
            "out of date".yellow(),
            local_package.version().bright_black(),
            latest_version.to_string().green(),
        );

        Ok(Some(PackageWithStatus::new(
            local_package,
            PackageStatus::OutOfDate,
            Some(latest_version),
        )))
    }

    pub fn check_all_packages(&self) -> anyhow::Result<Option<Vec<PackageWithStatus>>> {
        // Gather package status for each installed package.
        let statuses = self.get_package_statuses();
        let outdated_packages = statuses
            .iter()
            .filter(|pkg| pkg.is_out_of_date())
            .collect::<Vec<_>>();
        let outdated_package_padding = self.calculate_package_name_padding(&outdated_packages);

        // Check if all packages are up to date.
        if outdated_packages.is_empty() {
            println!("All packages are {}.", "up to date".green());
            return Ok(None);
        }

        // Print out-of-date packages.
        for package in &outdated_packages {
            let status_text = "out of date".yellow();
            let version_text = format!(
                "({} -> {})",
                package.package().version().bright_black(),
                package.latest_version.clone().unwrap().green()
            );
            println!(
                "{package_name:padding$} is {status_text} {version_text}",
                package_name = package.package().name().blue(),
                padding = outdated_package_padding,
            );
        }

        Ok(Some(
            outdated_packages.into_iter().cloned().collect::<Vec<_>>(),
        ))
    }

    pub fn update_package(&self, package_name: impl AsRef<str>) -> anyhow::Result<()> {
        let Ok(Some(package)) = self.check_package(package_name) else {
            return Ok(());
        };

        let local_package = package.package();
        let latest_version = package.latest_version.unwrap();

        match crate::cargo::update_package(
            local_package.name(),
            local_package.version().clone(),
            latest_version.clone(),
        ) {
            Ok(_) => {
                println!(
                    "{} {} from {} to {}.",
                    "Updated".green(),
                    local_package.name().blue(),
                    local_package.version().bright_black(),
                    latest_version.green(),
                );
            }
            Err(err) => {
                println!(
                    "{} to update package {}.",
                    "Failed".red(),
                    local_package.name().blue(),
                );
                println!(
                    "You may need to run {}.",
                    format!("cap update --locked {}", local_package.name()).bright_black()
                );
                return Err(err);
            }
        }

        Ok(())
    }

    pub fn update_all_packages(&self) -> anyhow::Result<()> {
        // Gather package status for each installed package.
        let Ok(Some(outdated_packages)) = self.check_all_packages() else {
            return Ok(());
        };

        println!(
            "{} {} {}...",
            "Updating".green(),
            outdated_packages.len(),
            crate::helper::pluralize("package", "packages", outdated_packages.len())
        );

        for package_with_status in &outdated_packages {
            let package_name = package_with_status.package().name();
            let local_version = package_with_status.package().version();
            let latest_version = package_with_status.latest_version.clone().unwrap();

            match crate::cargo::update_package(
                package_name,
                local_version.clone(),
                latest_version.clone(),
            ) {
                Ok(_) => {
                    println!(
                        "{} {} from {} to {}.",
                        "Updated".green(),
                        package_name.blue(),
                        local_version.bright_black(),
                        latest_version.green(),
                    );
                }
                Err(err) => {
                    println!(
                        "{} to update package {}.",
                        "Failed".red(),
                        package_name.blue(),
                    );
                    return Err(err);
                }
            }
        }

        Ok(())
    }

    fn get_package_statuses(&self) -> Vec<PackageWithStatus> {
        let mut packages_with_status = Vec::new();
        for package in self.packages.packages() {
            let Ok(latest_version) = self.registry.get_latest_version(package.name()) else {
                continue;
            };
            let is_up_to_date = latest_version <= *package.version();
            let status = if is_up_to_date {
                PackageStatus::UpToDate
            } else {
                PackageStatus::OutOfDate
            };
            packages_with_status.push(PackageWithStatus::new(
                package,
                status,
                Some(latest_version),
            ));
        }
        packages_with_status
    }

    fn calculate_package_name_padding<'p>(&self, packages: &[&'p PackageWithStatus<'p>]) -> usize {
        packages
            .iter()
            .map(|package| package.package().name().chars().count())
            .max()
            .unwrap_or(0)
    }
}
