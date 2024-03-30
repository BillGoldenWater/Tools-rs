use super::cached_regex::CachedRegex;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RegexReplace {
    i: CachedRegex,
    o: String,
}

impl RegexReplace {
    pub fn replace(&self, value: &str) -> anyhow::Result<String> {
        Ok(self.i.get()?.replace(value, &self.o).to_string())
    }
}
