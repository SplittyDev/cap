use curl::easy::Easy;

pub struct CratesIoClient {
    registry: crates_io::Registry,
}

impl CratesIoClient {
    pub fn new() -> Self {
        let index_url = "https://crates.io".to_string();
        let mut handle = Easy::new();
        handle
            .useragent("cap package manager (github.com/splittydev/cap)")
            .unwrap();
        Self {
            registry: crates_io::Registry::new_handle(index_url, None, handle, false),
        }
    }

    pub fn get_package_info(
        &mut self,
        package_name: impl AsRef<str>,
    ) -> anyhow::Result<crates_io::Crate> {
        let (crates, _) = self.registry.search(package_name.as_ref(), 1)?;
        let Some(crate_) = crates.into_iter().next() else {
            anyhow::bail!("Failed to find package on crates.io: {}", package_name.as_ref());
        };
        Ok(crate_)
    }
}
