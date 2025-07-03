use std::time::{Duration, Instant};

use humantime::format_duration;
use tracing::{debug, info};

use crate::{
    compresser::target::file::{self, FilterResult},
    config::Config,
    context::Context,
};

pub mod target;

pub fn run(ctx: &Context, config: &Config) -> anyhow::Result<()> {
    info!("running");

    let mut files = vec![];

    let mut filtered_out_by_time = 0_u64;
    for target_cfg in &config.target {
        for (path, size) in target::read_files(target_cfg)? {
            debug!("checking {:?}", path);
            let result = file::filter(target_cfg, &path)?;

            let keep = matches!(result, FilterResult::Process(_));
            if matches!(result, FilterResult::ByTime) {
                filtered_out_by_time += 1;
            }

            files.push((size, path, target_cfg, keep));
        }
    }

    let mut filtered_out = files.len();
    files.retain(|(_, _, _, keep)| *keep);
    filtered_out -= files.len();

    info!(
        "filterted out {filtered_out} files, {filtered_out_by_time} by time"
    );

    files.sort_by_key(|(size, ..)| *size);

    let total_size = files.iter().map(|(size, ..)| *size).sum::<u64>();
    let total_size_str = bytesize::ByteSize::b(total_size)
        .display()
        .si()
        .to_string()
        .into_boxed_str();
    let mut total_processed = 0;
    let mut total_compressed = 0;
    let start = Instant::now();

    let mut files_iter = files.iter().peekable();

    while let Some((size, path, target_cfg, _)) = files_iter.next() {
        info!("processing: ({}) {path:?}", format_bytes(*size));
        if let Some((size, path, ..)) = files_iter.peek() {
            info!("next: ({}) {path:?}", format_bytes(*size));
        }

        total_compressed += file::run(
            ctx,
            target_cfg,
            &config.ffmpeg_args_presets,
            path,
        )?;
        total_processed += size;

        #[expect(clippy::cast_precision_loss)]
        let percent = total_processed as f64 / total_size as f64 * 100.;

        #[expect(clippy::cast_precision_loss)]
        let compressed_size =
            total_compressed as f64 / total_processed as f64 * 100.;

        // eta
        let elapsed = start.elapsed();
        #[expect(clippy::cast_precision_loss)]
        let speed = total_processed as f64 / elapsed.as_secs_f64();
        let remaining = total_size - total_processed;
        #[expect(clippy::cast_precision_loss)]
        let eta = if remaining > 0 {
            remaining as f64 / speed
        } else {
            0.
        };

        info!(
            "processed: {}/{}({percent:.2}%), compressed size: {compressed_size:.2}%",
            format_bytes(total_processed),
            total_size_str,
        );
        info!(
            "elpased: {}, eta: +{}",
            format_duration(elapsed),
            format_duration(Duration::from_secs_f64(eta)),
        );
    }

    Ok(())
}

fn format_bytes(size: u64) -> bytesize::Display {
    bytesize::ByteSize::b(size).display().si()
}
