use std::fmt::Display;

use anyhow::Context;
use log::warn;

use super::{
    BinaryInvocationScraper, CargoMetadataScraper, LocalPackageMetadataScraper, Package,
    PackageExecutable,
};

#[derive(Debug)]
pub enum PackageFormatting {
    Long,
    Short,
}

#[derive(Debug)]
pub struct PackageTree {
    packages: Vec<Package>,
}

impl PackageTree {
    pub fn build() -> anyhow::Result<Self> {
        // Scrape package metadata
        let package_map = CargoMetadataScraper::scrape()
            .or_else(|_| {
                warn!("Failed to scrape Cargo metadata. Falling back to binary scraping.");
                BinaryInvocationScraper::scrape()
            })
            .context("Unable to scrape package metadata.")?;

        // Build full package index
        let mut packages = package_map
            .into_iter()
            .map(|(package, binaries)| {
                Package::new(
                    package.name().to_string(),
                    package.version().clone(),
                    binaries,
                )
            })
            .collect::<Vec<_>>();

        // Sort packages
        packages.sort();

        Ok(Self { packages })
    }

    pub fn packages(&self) -> impl Iterator<Item = &Package> {
        self.packages.iter()
    }

    pub fn get(&self, package_name: impl AsRef<str>) -> Option<&Package> {
        self.packages
            .iter()
            .find(|package| package.name() == package_name.as_ref())
    }

    pub fn print(&self, formatting: PackageFormatting) {
        match formatting {
            PackageFormatting::Long => {
                for package in self.packages() {
                    println!("{} (v{})", package.name(), package.version());
                    for binary in package.binaries() {
                        println!("  {}", binary.name());
                    }
                }
            }
            PackageFormatting::Short => {
                for package in self.packages() {
                    let binaries = package
                        .binaries()
                        .iter()
                        .map(PackageExecutable::name)
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("{} (v{}): {}", package.name(), package.version(), binaries);
                }
            }
        }
    }
}

impl Display for PackageTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for package in self.packages() {
            writeln!(f, "{} (v{})", package.name(), package.version())?;
            for binary in package.binaries() {
                writeln!(f, "  {}", binary.name())?;
            }
        }
        Ok(())
    }
}
