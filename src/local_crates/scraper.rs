use std::collections::HashMap;

use super::{PackageExecutable, PackageKey};

pub trait LocalPackageMetadataScraper {
    fn scrape() -> anyhow::Result<HashMap<PackageKey, Vec<PackageExecutable>>>;
}
