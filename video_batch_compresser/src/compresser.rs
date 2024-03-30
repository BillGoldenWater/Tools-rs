use tracing::info;

use crate::config::Config;

pub mod target;

pub fn run(config: Config) -> anyhow::Result<()> {
    info!("running");

    for target_cfg in config.target {
        target::run(target_cfg)?;
    }

    Ok(())
}
