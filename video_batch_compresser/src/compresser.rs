use tracing::info;

use crate::{compresser::target::file, config::Config, context::Context};

pub mod target;

pub fn run(ctx: &Context, config: &Config) -> anyhow::Result<()> {
    info!("running");

    let mut files = vec![];

    for target_cfg in &config.target {
        for (path, size) in target::read_files(target_cfg)? {
            files.push((size, path, target_cfg));
        }
    }

    files.sort_by_key(|(size, ..)| *size);

    for (size, path, target) in files {
        info!("file to process: ({size}){path:?}");
        file::run(ctx, target, &path)?;
    }

    Ok(())
}
