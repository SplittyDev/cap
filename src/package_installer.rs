use owo_colors::OwoColorize;

use crate::{CratesRegistry, PackageTree};

pub struct PackageInstaller<'a> {
    registry: &'a CratesRegistry,
    packages: &'a PackageTree,
}

impl<'a> PackageInstaller<'a> {
    pub fn new(registry: &'a CratesRegistry, packages: &'a PackageTree) -> Self {
        Self { registry, packages }
    }

    pub fn install_package(
        &self,
        package_name: impl AsRef<str>,
        locked: bool,
        forced: bool,
        nightly: bool,
    ) -> anyhow::Result<()> {
        let package_name = package_name.as_ref();

        if !forced {
            if let Some(local_package) = self.packages.get(package_name) {
                println!(
                    "Package {} is {}.",
                    local_package.name().blue(),
                    "already installed".green()
                );
                return Ok(());
            }
        }

        let Ok(latest_version) = self.registry.get_latest_version(package_name) else {
            println!("Package {} is {}.", package_name.blue(), "not available on crates.io".red());
            return Ok(());
        };

        match crate::cargo::install_package(
            package_name,
            latest_version.clone(),
            locked,
            forced,
            nightly,
        ) {
            Ok(_) => {
                println!(
                    "{} {} {}.",
                    "Installed".green(),
                    package_name.blue(),
                    latest_version.to_string().bright_black(),
                );
            }
            Err(err) => {
                println!(
                    "{} to install package {} {}.",
                    "Failed".red(),
                    package_name.blue(),
                    latest_version.to_string().bright_black()
                );
                return Err(err);
            }
        }

        Ok(())
    }

    pub fn uninstall_package(&self, package_name: impl AsRef<str>) -> anyhow::Result<()> {
        let package_name = package_name.as_ref();

        let Some(local_package) = self.packages.get(package_name) else {
            println!("Package {} is {}.", package_name.blue(), "not installed".red());
            return Ok(());
        };

        println!(
            "Uninstalling package {} {}.",
            package_name.blue(),
            local_package.version().to_string().bright_black()
        );

        crate::cargo::uninstall_package(package_name)?;

        Ok(())
    }
}
