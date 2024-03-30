use std::sync::OnceLock;

use anyhow::Context as _;
use regex::Regex;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct CachedRegex(String, #[serde(skip)] OnceLock<Regex>);

impl CachedRegex {
    pub fn get(&self) -> anyhow::Result<&Regex> {
        let regex = if let Some(regex) = self.1.get() {
            regex
        } else {
            let regex = Regex::new(&self.0).context("failed to parse regex")?;
            self.1.get_or_init(|| regex)
        };
        Ok(regex)
    }
}
