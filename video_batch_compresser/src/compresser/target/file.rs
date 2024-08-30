use std::path::PathBuf;

use anyhow::{anyhow, Context as _};
use chrono::{Datelike, TimeDelta, Utc};
use tracing::{debug, info};

use crate::{config::target::Target, context::Context, ffmpeg};

pub fn run(ctx: &Context, target_cfg: &Target, file: PathBuf) -> anyhow::Result<()> {
    let file_name = file
        .file_name()
        .ok_or_else(|| anyhow!("failed to get file name of the entry"))?
        .to_str()
        .ok_or_else(|| anyhow!("the file name isn't valid utf8"))?;

    if !target_cfg.filter.get()?.is_match(file_name) {
        info!("filtered out, by regex");
        return Ok(());
    }

    let time = target_cfg
        .time_extractor
        .replace(file_name)
        .context("failed to extract time")?;
    let time = chrono::DateTime::parse_from_rfc3339(&time)
        .with_context(|| format!("failed to parse extracted time by rfc3339: {:?}", time))?;

    let delta = TimeDelta::try_days(target_cfg.filter_before as i64)
        .ok_or_else(|| anyhow!("invalid duration"))?;
    let time_added = time
        .checked_add_signed(delta)
        .ok_or_else(|| anyhow!("failed to do (time + filter_before)"))?;
    if time_added.ge(&Utc::now()) {
        info!("filtered out, by time");
        return Ok(());
    }

    let mut mark_path = file.clone();
    mark_path.set_file_name(format!("{}.compressed_mark", file_name));
    if mark_path
        .try_exists()
        .context("failed to check is marker exists")?
    {
        info!("filtered out, by marker");
        return Ok(());
    }

    info!("input: {}", file_name);

    let mut associated_files = vec![];
    for extractor in &target_cfg.associated_file_extractor {
        let file_name = extractor
            .replace(file_name)
            .context("failed to extract associated file")?;
        let path = target_cfg.input.join(&file_name);

        if !path
            .try_exists()
            .context("failed to check is associated file exists")?
        {
            debug!("skip {file_name}, isn't exists");
            continue;
        }

        if !path.is_file() {
            debug!("skip {file_name}, isn't file");
            continue;
        }

        associated_files.push(path);
    }

    info!("associated files: {associated_files:?}");

    let temp = ctx.cwd.join("video_batch_compresser.temp.mkv");
    if temp
        .try_exists()
        .context("failed to check is temp file exists")?
    {
        return Err(anyhow!("has things exists at {temp:?}"));
    }

    info!("compressing");
    ffmpeg::run(file.as_os_str(), &target_cfg.ffmpeg_args, temp.as_os_str())
        .context("failed to compress")?;

    let mut output_file_name = PathBuf::from(file_name);
    output_file_name.set_extension("mkv");

    let output_base = target_cfg
        .output
        .join(format!("{:0>4}-{:0>2}", time.year(), time.month()));
    if !output_base
        .try_exists()
        .context("failed to check is output dir exists")?
    {
        std::fs::create_dir_all(&output_base).context("failed to create output dir")?;
    }

    let output = output_base.join(output_file_name);

    info!("output: {:?}", output);

    if output
        .try_exists()
        .context("failed to check is output path already has things exists")?
    {
        return Err(anyhow!("has things exists at the target location"));
    }

    info!("moving compressed file");
    std::fs::rename(temp, output).context("failed to move compressed file to the output path")?;

    for f in associated_files {
        let output = output_base.join(
            f.file_name()
                .ok_or_else(|| anyhow!("failed to get file name of associated file"))?,
        );

        if output
            .try_exists()
            .context("failed to check is output path already has things exists")?
        {
            return Err(anyhow!("has things exists at the target location"));
        }

        info!("copying associated file {:?} to {:?}", f, output);

        std::fs::copy(f, output)
            .context("failed to copy the associated file to the output location")?;
    }

    info!("done, creating marker");
    std::fs::write(mark_path, "").context("failed to create compressed mark")?;

    Ok(())
}
