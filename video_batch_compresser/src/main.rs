#![warn(missing_debug_implementations)]

use std::{fs, path::PathBuf};

use anyhow::Context;
use config::Config;

pub mod compresser;
pub mod config;
pub mod context;
pub mod ffmpeg;

#[derive(Debug, argh::FromArgs)]
/// a simple tool for batch compress media files for archive
pub struct Args {
    /// path to config file, default to compress.toml in the current working directory
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
    let ctx = context::Context::new().context("failed to contruct Context")?;

    let config_path = args.config.unwrap_or(ctx.cwd.join("compress.toml"));
    let config = fs::read_to_string(config_path).context("failed to read config file")?;
    let config: Config = toml::from_str(&config).context("failed to parse config file")?;

    compresser::run(&ctx, config)?;

    Ok(())
}
