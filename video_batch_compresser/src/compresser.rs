use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::{Duration, Instant},
};

use anyhow::Context as _;
use humantime::format_duration;
use tracing::{debug, info};

use crate::{
    compresser::target::file::{self, FilterResult},
    config::Config,
    context::Context,
};

pub mod target;

#[expect(clippy::too_many_lines)]
pub fn run(ctx: &mut Context, config: &Config) -> anyhow::Result<()> {
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

    if let Some(speed) = ctx.speed_cache.get("__all__").copied() {
        #[expect(clippy::cast_precision_loss)]
        let eta = total_size as f64 / speed;
        info!("eta: +{}", format_duration(Duration::from_secs_f64(eta)));
    }

    let start = Instant::now();
    let mut files_iter = files.iter().peekable();

    while let Some((size, path, target_cfg, _)) = files_iter.next() {
        let speed_cache_id = {
            let ffmpeg_args_id =
                if let Some(id) = &target_cfg.ffmpeg_args_id {
                    id.to_string()
                } else {
                    let args = target_cfg
                        .get_ffmpeg_args(&config.ffmpeg_args_presets)?;
                    let mut hasher = DefaultHasher::new();
                    args.hash(&mut hasher);
                    hasher.finish().to_string()
                };

            let mut hasher = DefaultHasher::new();
            target_cfg.input.hash(&mut hasher);
            let target_id = hasher.finish().to_string();

            format!("{ffmpeg_args_id}-{target_id}")
        };
        let cached_speed = ctx
            .speed_cache
            .get(&speed_cache_id)
            .copied()
            .or_else(|| ctx.speed_cache.get("__all__").copied());

        info!("processing: ({}) {path:?}", format_bytes(*size));
        if let Some(speed) = cached_speed {
            #[expect(clippy::cast_precision_loss)]
            let eta = *size as f64 / speed;
            let eta = Duration::from_secs_f64(eta);
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let speed = speed as u64;
            info!(
                "avg speed and eta for this file: {}/s +{}",
                format_bytes(speed),
                format_duration(eta)
            );
        }
        if let Some((size, path, ..)) = files_iter.peek() {
            info!("next: ({}) {path:?}", format_bytes(*size));
        }

        // run
        let file_start = Instant::now();
        total_compressed += file::run(
            ctx,
            target_cfg,
            &config.ffmpeg_args_presets,
            path,
        )?;
        #[expect(clippy::cast_precision_loss)]
        {
            let elasped = file_start.elapsed().as_secs_f64();
            let speed = *size as f64 / elasped;
            let speed_prev = cached_speed.unwrap_or(speed);
            let speed = ema(speed, speed_prev, 1. / 32.);
            ctx.speed_cache.insert(speed_cache_id, speed);
        }
        total_processed += size;

        #[expect(clippy::cast_precision_loss)]
        let percent = total_processed as f64 / total_size as f64 * 100.;

        #[expect(clippy::cast_precision_loss)]
        let compressed_size =
            total_compressed as f64 / total_processed as f64 * 100.;

        // eta
        let elapsed = start.elapsed();
        #[expect(clippy::cast_precision_loss)]
        let speed = {
            let speed = total_processed as f64 / elapsed.as_secs_f64();
            let speed_prev =
                ctx.speed_cache.get("__all__").copied().unwrap_or(speed);
            let speed = ema(speed, speed_prev, 1. / 32.);
            ctx.speed_cache.insert("__all__".into(), speed);
            speed
        };
        #[expect(clippy::cast_precision_loss)]
        let eta = {
            let remaining = total_size - total_processed;
            if remaining > 0 {
                remaining as f64 / speed
            } else {
                0.
            }
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

        ctx.save().context("ctx.save()")?;
    }

    Ok(())
}

fn format_bytes(size: u64) -> bytesize::Display {
    bytesize::ByteSize::b(size).display().si()
}

/// <https://en.wikipedia.org/wiki/Exponential_smoothing>
fn ema(value: f64, prev: f64, alpha: f64) -> f64 {
    alpha.mul_add(value, (1. - alpha) * prev)
}
