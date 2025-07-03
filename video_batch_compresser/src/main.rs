#![warn(missing_debug_implementations)]
#![warn(clippy::pedantic, clippy::nursery)]
// #![clippy::too_many_line_threshold = 60]
#![allow(clippy::default_trait_access)]

use std::{fs, path::PathBuf};

use anyhow::Context;
use config::Config;
use tracing_subscriber::EnvFilter;

mod compresser;
mod config;
mod context;
mod ffmpeg;

#[derive(Debug, argh::FromArgs)]
/// a simple tool for batch compress media files for archive
pub struct Args {
    /// path to config file, default to compress.toml in the current working directory
    #[argh(option, short = 'c')]
    config: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = argh::from_env::<Args>();
    let ctx =
        context::Context::new().context("failed to contruct Context")?;

    let config_path =
        args.config.unwrap_or_else(|| ctx.cwd.join("compress.toml"));
    let config = fs::read_to_string(config_path)
        .context("failed to read config file")?;
    let config: Config =
        toml::from_str(&config).context("failed to parse config file")?;

    compresser::run(&ctx, &config)?;

    Ok(())
}
