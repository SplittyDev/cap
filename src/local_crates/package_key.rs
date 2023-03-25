#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageKey {
    name: String,
    version: semver::Version,
}

impl PackageKey {
    pub fn new(name: impl ToString, version: semver::Version) -> Self {
        Self {
            name: name.to_string(),
            version,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &semver::Version {
        &self.version
    }
}
