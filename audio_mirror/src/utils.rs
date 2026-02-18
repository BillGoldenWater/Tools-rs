use anyhow::Context as _;
use cpal::{
    Device, Host, StreamConfig, SupportedStreamConfig,
    traits::{DeviceTrait as _, HostTrait as _},
};

pub fn get_default_output(
    host: &Host,
) -> anyhow::Result<(Device, SupportedStreamConfig, StreamConfig)> {
    let device =
        host.default_output_device().context("get default output")?;
    let supported_config = device
        .default_output_config()
        .context("get default output config")?;
    let mut config = supported_config.config();
    config.buffer_size = match supported_config.buffer_size() {
        cpal::SupportedBufferSize::Range { min, .. } => {
            cpal::BufferSize::Fixed(*min)
        }
        cpal::SupportedBufferSize::Unknown => {
            cpal::BufferSize::Fixed(128)
        }
    };

    Ok((device, supported_config, config))
}
