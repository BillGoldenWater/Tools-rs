use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::Context as _;

#[derive(Debug)]
pub struct Context {
    pub cwd: PathBuf,
    pub speed_cache: HashMap<String, f64>,
}

impl Context {
    pub fn new() -> anyhow::Result<Self> {
        let cwd = std::env::current_dir()
            .context("failed to get current working directory")?;
        let speed_cache = {
            let cache = fs::read_to_string(cwd.join("speed_cache.toml"))
                .context("read compression speed cache")?;
            toml::from_str(&cache).context("parse speed cache")?
        };

        Ok(Self { cwd, speed_cache })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let speed_cache = toml::to_string_pretty(&self.speed_cache)
            .context("toml::to_string_pretty(speed_cache)")?;
        fs::write(self.cwd.join("speed_cache.toml"), speed_cache)
            .context("write compression speed cache")?;

        Ok(())
    }
}
