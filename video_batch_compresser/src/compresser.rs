use std::time::{Duration, Instant};

use humantime::format_duration;
use tracing::info;

use crate::{compresser::target::file, config::Config, context::Context};

pub mod target;

pub fn run(ctx: &Context, config: &Config) -> anyhow::Result<()> {
    info!("running");

    let mut files = vec![];

    for target_cfg in &config.target {
        for (path, size) in target::read_files(target_cfg)? {
            info!("preprocessing {:?}", path);
            let keep = file::filter(target_cfg, &path)?.is_some();
            files.push((size, path, target_cfg, keep));
        }
    }

    files.retain(|(_, _, _, keep)| *keep);
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

    for (size, path, target_cfg, _) in files {
        info!(
            "processing: ({}) {path:?}",
            bytesize::ByteSize::b(size).display().si()
        );
        total_compressed += file::run(ctx, target_cfg, &path)?;
        total_processed += size;

        #[expect(clippy::cast_precision_loss)]
        let percent = total_processed as f64 / total_size as f64 * 100.;

        #[expect(clippy::cast_precision_loss)]
        let compress_ratio =
            total_compressed as f64 / total_processed as f64 * 100.;

        // eta
        let elapsed = start.elapsed();
        #[expect(clippy::cast_precision_loss)]
        let speed = total_processed as f64 / elapsed.as_secs_f64();
        let to_process = total_size - total_processed;
        #[expect(clippy::cast_precision_loss)]
        let eta = if to_process > 0 {
            to_process as f64 / speed
        } else {
            0.
        };

        info!(
            "processed: {}/{}({percent:.2}%), compress_ratio: {compress_ratio:.2}%",
            bytesize::ByteSize::b(total_processed).display().si(),
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
