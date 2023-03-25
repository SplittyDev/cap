use super::PackageExecutable;

/// A package including name, version, and binaries.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Package {
    name: String,
    version: semver::Version,
    binaries: Vec<PackageExecutable>,
}

impl Package {
    pub fn new(name: String, version: semver::Version, binaries: Vec<PackageExecutable>) -> Self {
        Self {
            name,
            version,
            binaries,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &semver::Version {
        &self.version
    }

    pub fn binaries(&self) -> &[PackageExecutable] {
        &self.binaries
    }
}
