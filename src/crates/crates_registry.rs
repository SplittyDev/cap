use anyhow::Context;
use crates_index::Index;

// const CRATES_IO_SPARSE_INDEX_URL: &str = "sparse+https://index.crates.io/";

pub struct CratesRegistry {
    index: Index,
}

impl CratesRegistry {
    pub fn new() -> Self {
        let index = Index::new_cargo_default().unwrap();

        Self { index }
    }

    /// Update the crates.io index.
    ///
    /// This can be slow, so it should only be done when necessary.
    pub fn _update_index(&mut self) -> anyhow::Result<()> {
        self.index
            .update()
            .context("Failed to update crates.io index")
    }

    /// Get the latest version of a crate.
    ///
    /// If the crate has no normal versions, the highest version will be returned,
    /// even if it's a pre-release or yanked version. This is done in order to guarantee
    /// that some version can always be returned.
    pub fn get_latest_version(
        &self,
        crate_name: impl AsRef<str>,
    ) -> anyhow::Result<semver::Version> {
        let crate_name = crate_name.as_ref();
        let crate_ = self
            .index
            .crate_(crate_name)
            .context(format!("Failed to find crate: {}", crate_name))?;
        let latest_version = crate_
            .highest_normal_version()
            .unwrap_or_else(|| crate_.highest_version());
        semver::Version::parse(latest_version.version())
            .context(format!("Failed to parse version of crate: {}", crate_name))
    }
}

impl Default for CratesRegistry {
    fn default() -> Self {
        Self::new()
    }
}
