#![warn(missing_debug_implementations)]

use std::{fs, path::PathBuf, str::FromStr};

use anyhow::Context;
use config::Config;

pub mod compresser;
pub mod config;
pub mod ffmpeg;

#[derive(Debug, argh::FromArgs)]
/// a simple tool for batch compress media files for archive
pub struct Args {
    /// path to config file, default to ./compress.toml
    #[argh(option, short = 'c')]
    config: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    {
        tracing_subscriber::fmt::init();
    }

    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    let args = argh::from_env::<Args>();
    let config_path = args.config.unwrap_or(
        PathBuf::from_str("./compress.toml").context("failed to construct default path")?,
    );
    let config = fs::read_to_string(config_path).context("failed to read config file")?;
    let config: Config = toml::from_str(&config).context("failed to parse config file")?;

    compresser::run(config)?;

    Ok(())
}
