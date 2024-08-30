use std::path::PathBuf;

use anyhow::Context as _;

#[derive(Debug)]
pub struct Context {
    pub cwd: PathBuf,
}

impl Context {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            cwd: std::env::current_dir().context("failed to get current working directory")?,
        })
    }
}
