use anyhow::Context as _;
use cpal::{
    Device, Host, StreamConfig, SupportedStreamConfig,
    traits::{DeviceTrait as _, HostTrait as _},
};

use crate::Args;

pub fn get_default_output(
    host: &Host,
    args: &Args,
) -> anyhow::Result<(Device, SupportedStreamConfig, StreamConfig)> {
    let device =
        host.default_output_device().context("get default output")?;
    let supported_config = device
        .default_output_config()
        .context("get default output config")?;
    let mut config = supported_config.config();
    config.buffer_size = match supported_config.buffer_size() {
        cpal::SupportedBufferSize::Range { min, max } => {
            cpal::BufferSize::Fixed(args.buffer_size.clamp(*min, *max))
        }
        cpal::SupportedBufferSize::Unknown => {
            cpal::BufferSize::Fixed(args.buffer_size)
        }
    };
    tracing::debug!("using buffer size: {:?}", config.buffer_size);

    Ok((device, supported_config, config))
}
