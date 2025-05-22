use std::{
    io::{BufReader, Read},
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, bail};
use glam::{DVec2, USizeVec2, UVec2};
use image::{GenericImageView, ImageBuffer, Rgba};
use img_hash::{HashAlg, HasherConfig};
use itertools::Itertools;

fn main() -> anyhow::Result<()> {
    let video_fps_as = 4;
    let video_dims = USizeVec2::new(1920, 1080);

    let pattern_dir = "../pattern/";
    let source_dir = "../source/";
    let output_dir = "../stage1/";

    let pattern_dims = DVec2::new(250. / 1920., 250. / 1080.);
    let pattern_pos = DVec2::new(
        0.5 - pattern_dims.x / 2.0,
        0.5 - (1080. - 975.) / 1080. / 2.0 - pattern_dims.y / 2.0,
    );

    // let target_dims = DVec2::new(720. / 1920., 150. / 1080.);
    // let target_pos = DVec2::new(
    //     0.5 - target_dims.x / 2.0,
    //     1. - 85. / 1080. - target_dims.y - 1. / 1080.,
    // );

    let mut patterns = Vec::<ImageBuffer<Rgba<u8>, Vec<u8>>>::new();

    for entry in std::fs::read_dir(pattern_dir)
        .context("failed to read pattern dir")?
    {
        let entry = entry.context("failed to read pattern dir entry")?;
        let img = image::open(entry.path())
            .context("failed to read pattern image")?;
        let img_dims =
            UVec2::from_array(img.dimensions().into()).as_dvec2();
        let pattern_pos = (pattern_pos * img_dims).as_uvec2();
        let pattern_dims = (pattern_dims * img_dims).as_uvec2();
        let pattern = img.crop_imm(
            pattern_pos.x,
            pattern_pos.y,
            pattern_dims.x,
            pattern_dims.y,
        );
        patterns.push(pattern.to_rgba8());
    }

    let (pattern_pos, pattern_dims) = {
        let video_dims = video_dims.as_dvec2();
        (
            (pattern_pos * video_dims).as_uvec2(),
            (pattern_dims * video_dims).as_uvec2(),
        )
    };

    let hasher = HasherConfig::new()
        .hash_size(8, 8)
        .hash_alg(HashAlg::Gradient)
        .to_hasher();
    let patterns = patterns
        .into_iter()
        .map(|pattern| hasher.hash_image(&pattern))
        .collect::<Vec<_>>();

    let sources = std::fs::read_dir(source_dir)
        .context("failed to read source dir")?
        .collect::<Result<Vec<_>, _>>()
        .context("failed to read source dir entries")?
        .into_iter()
        .sorted_by_key(|it| it.path());

    let mut frame_data_buffer =
        vec![0_u8; video_dims.x * video_dims.y * 4];
    let video_dims = video_dims.as_uvec2();

    let mut output_count = 1;
    for entry in sources {
        let input = entry.path();

        let mut ffmpeg = Command::new("ffmpeg")
            .args(["-i", input.to_str().expect("expect utf8 compatible")])
            .args(["-r", &format!("{video_fps_as}")])
            .args(["-s", &format!("{}x{}", video_dims.x, video_dims.y)])
            .args(["-pix_fmt", "rgba"])
            .args(["-f", "rawvideo"])
            .arg("-")
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("failed to spawn ffmpeg")?;
        let ffmpeg_out =
            ffmpeg.stdout.take().context("expect stdout from ffmpeg")?;
        let mut ffmpeg_out = BufReader::new(ffmpeg_out);

        let mut frame_count = 1;
        let mut failed_count = 0;
        let ffmpeg_status = loop {
            if let Some(status) = ffmpeg
                .try_wait()
                .context("failed to check ffmpeg status")?
            {
                break status;
            }

            let result = ffmpeg_out.read_exact(&mut frame_data_buffer);
            if let Err(err) = result {
                failed_count += 1;
                println!("failed to read frame data from ffmpeg: {err}");
                if failed_count > 100 {
                    bail!(
                        "failed count > 100 and ffmpeg still not terminate"
                    );
                }
                continue;
            }

            let frame = ImageBuffer::<Rgba<u8>, _>::from_raw(
                video_dims.x,
                video_dims.y,
                frame_data_buffer,
            )
            .context("failed to create image from raw data")?;

            let pattern_area = frame.view(
                pattern_pos.x,
                pattern_pos.y,
                pattern_dims.x,
                pattern_dims.y,
            );

            let hash = hasher.hash_image(&pattern_area.to_image());
            let dist = patterns
                .iter()
                .map(|it| it.dist(&hash))
                .min()
                .expect("expect non empty");
            if dist < 6 {
                println!("{frame_count: >10} {dist:?}");

                frame
                    .save(
                        Path::new(output_dir)
                            .join(format!("{output_count}.png")),
                    )
                    .context("failed to save output")?;
                output_count += 1;
            }

            frame_data_buffer = frame.into_raw();
            frame_count += 1;
        };

        if !ffmpeg_status.success() {
            bail!(
                "ffmpeg finished with non zero exit code: {ffmpeg_status}"
            );
        } else if failed_count > 0 {
            println!(
                "ffmpeg finished with success exit status, but failed count > 0, assuming no fatal error"
            );
        }
    }

    Ok(())
}
