use tracing::info;

use crate::{config::Config, context::Context};

pub mod target;

pub fn run(ctx: &Context, config: Config) -> anyhow::Result<()> {
    info!("running");

    for target_cfg in config.target {
        target::run(ctx, &target_cfg)?;
    }

    Ok(())
}
