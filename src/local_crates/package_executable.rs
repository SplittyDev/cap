#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackageExecutable {
    name: String,
}

impl PackageExecutable {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
