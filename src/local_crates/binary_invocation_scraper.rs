use anyhow::Context;
use log::trace;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use std::{collections::HashMap, process::Command};

use super::{LocalPackageMetadataScraper, PackageExecutable, PackageKey};

/// Find the first semver version in a string.
///
/// This is NOT a correct semver parser, it's just a naive
/// heuristic that tries to find a version in a string.
fn find_version(str: &str) -> Option<semver::Version> {
    let regex = regex::Regex::new(r"v?(\d+\.\d+(?:\.\d+)?)").unwrap();
    for line in str.lines() {
        for chunk in line.split_whitespace() {
            let Some(captures) = regex.captures(chunk) else { continue };
            let Some(version_capture) = captures.get(1) else { continue };
            let Ok(version) = semver::Version::parse(version_capture.as_str()) else { continue };
            return Some(version);
        }
    }
    trace!("Unable to find version in string: {}", str);
    None
}

/// A naive package tree scraper that invokes local binaries
/// to get information about installed packages.
///
/// This is a lot slower than the cargo metadata-based approach used
/// in the `CargoMetadataScraper` and it's only intended to be used as a
/// fallback when the cargo metadata-based approach fails.
pub struct BinaryInvocationScraper;

impl LocalPackageMetadataScraper for BinaryInvocationScraper {
    fn scrape() -> anyhow::Result<HashMap<PackageKey, Vec<PackageExecutable>>> {
        // Get cargo bin dir
        let dir = home::cargo_home()
            .map(|pb| pb.join("bin"))
            .expect("Unable to find cargo bin dir.");

        // Read cargo bin dir
        std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    // Skip invalid entries
                    .filter_map(|entry| entry.ok())
                    // Only keep entries that are files
                    .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or_default())
                    // Parallelize iterator
                    .par_bridge()
                    // Build hashmap of packages and executables
                    .fold(HashMap::new, |mut map, entry| {
                        if let Ok(output) =
                            Command::new(entry.file_name()).arg("--version").output()
                        {
                            // Get stdout as UTF-8 string
                            let output_str = String::from_utf8(output.stdout).unwrap_or_default();

                            // Get iterator over words of first line
                            let mut parts_iter = output_str
                                .lines()
                                .next()
                                .unwrap_or_default()
                                .split_whitespace();

                            // Assume that the first word of the first line is the package name
                            let package_name = parts_iter.next();

                            // Try to find a version in the output
                            let version = find_version(&output_str);

                            if let (Some(package_name), Some(version)) = (package_name, version) {
                                // Build package info from version output
                                let package_key =
                                    PackageKey::new(package_name.to_lowercase(), version);

                                // Build executable info from file name
                                let executable = PackageExecutable::new(
                                    entry.file_name().to_string_lossy().to_lowercase(),
                                );

                                // Add package and executable to the hashmap
                                map.entry(package_key)
                                    .or_insert_with(Vec::new)
                                    .push(executable);
                            }
                        }
                        map
                    })
                    // Merge the hashmaps from each thread into one
                    .reduce(HashMap::new, |mut map1, map2| {
                        for (package, executables) in map2 {
                            map1.entry(package)
                                .or_insert_with(Vec::new)
                                .extend(executables);
                        }
                        map1
                    })
            })
            .context("Unable to read cargo bin dir.")
    }
}
