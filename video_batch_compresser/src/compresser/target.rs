use anyhow::{anyhow, Context as _};
use tracing::info;

use crate::{config::target::Target, context::Context};

pub mod file;

pub fn run(ctx: &Context, target: Target) -> anyhow::Result<()> {
    info!(
        "compressing target: {:?}, output to: {:?}",
        target.input, target.output
    );

    if !target.input.is_dir() {
        return Err(anyhow!("invalid input path, expect directory"));
    }

    if !target
        .output
        .try_exists()
        .context("failed to check is output dir exists")?
    {
        std::fs::create_dir_all(&target.output).context("failed to create output dir")?;
    }

    if !target.output.is_dir() {
        return Err(anyhow!("invalid output path, expect directory"));
    }

    for entry in target
        .input
        .read_dir()
        .context("failed to read input directory")?
    {
        let entry = entry.context("failed to access entry in input directory")?;
        let path = entry.path();

        info!("processing {:?}", path);
        let metadata = entry
            .metadata()
            .context("failed to read metadata of entry")?;

        if !metadata.is_file() {
            info!("isn't file, skipping");
            continue;
        }

        file::run(ctx, &target, path)?;
    }

    Ok(())
}
