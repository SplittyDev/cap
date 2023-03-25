use owo_colors::OwoColorize;

use crate::{CratesRegistry, Package, PackageTree};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum PackageStatus {
    UpToDate,
    OutOfDate,
}

#[derive(Debug)]
struct PackageWithStatus<'a> {
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

    pub fn update_package(&self, package_name: impl AsRef<str>) -> anyhow::Result<()> {
        let Some(local_package) = self.packages.get(&package_name) else {
            println!("Package {} is {}.", package_name.as_ref().blue(), "not installed".red());
            return Ok(());
        };
        let Ok(latest_version) = self.registry.get_latest_version(local_package.name()) else {
            println!("Package {} is {}.", local_package.name().blue(), "not available on crates.io".red());
            return Ok(());
        };
        if latest_version <= *local_package.version() {
            println!(
                "Package {} is {}.",
                local_package.name().blue(),
                "up to date".green()
            );
            return Ok(());
        }
        println!(
            "Updating {} from {} to {}...",
            local_package.name().blue(),
            local_package.version().bright_black(),
            latest_version.green()
        );
        crate::cargo::update_package(local_package.name(), latest_version)?;
        Ok(())
    }

    pub fn update_all_packages(&self) -> anyhow::Result<()> {
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
            return Ok(());
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

        // Update packages.
        println!("Updating {} packages...", outdated_packages.len());
        for package in &outdated_packages {
            let package_name = package.package().name();
            let latest_version = package.latest_version.clone().unwrap();
            println!(
                "Updating {} from {} to {}...",
                package_name.blue(),
                package.package().version().bright_black(),
                latest_version.green()
            );
            crate::cargo::update_package(package.package().name(), latest_version)?;
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
