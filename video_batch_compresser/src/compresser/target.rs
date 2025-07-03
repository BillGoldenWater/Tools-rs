use std::path::PathBuf;

use anyhow::{Context as _, anyhow};
use tracing::info;

use crate::config::target::Target;

pub mod file;

pub fn read_files(
    target: &Target,
) -> anyhow::Result<Vec<(PathBuf, u64)>> {
    info!(
        "reading target: {:?}, will output to: {:?}",
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
        std::fs::create_dir_all(&target.output)
            .context("failed to create output dir")?;
    }

    if !target.output.is_dir() {
        return Err(anyhow!("invalid output path, expect directory"));
    }

    target
        .input
        .read_dir()
        .context("failed to read input directory")?
        .map(|entry| {
            let entry = entry
                .context("failed to access entry in input directory")?;

            let path = entry.path();

            let metadata = entry
                .metadata()
                .context("failed to read metadata of entry")?;

            if !metadata.is_file() {
                info!("isn't file, skipping");
                return Ok(None);
            }
            let file_size = metadata.len();

            Ok(Some((path, file_size)))
        })
        .filter_map(|it| match it {
            Ok(it) => it.map(Ok),
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<Vec<_>, _>>()
}
