#![warn(missing_debug_implementations)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::default_trait_access)]

use std::{ops::Deref, path::PathBuf, str::FromStr};

use anyhow::{Context, anyhow, bail};
use argh::FromArgs;
use color::{Oklch, OpaqueColor, Srgb};
use glam::{DVec3, DVec4, Vec4Swizzles};
use image::{ImageBuffer, Rgba};
use rayon::prelude::ParallelIterator;
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

#[derive(Debug, Clone, FromArgs)]
/// a tool for "blend" two different images,
/// so that when viewed against a white background,
/// the "bright" image is visible, and vice versa
struct Args {
    #[argh(positional)]
    /// bright part of final image
    bright_path: PathBuf,
    #[argh(positional)]
    /// dark part of final image
    dark_path: PathBuf,
    #[argh(positional)]
    /// final image target file path, would overwrite!
    out_path: PathBuf,
    #[argh(option)]
    /// how much contrast capacity of final image would the bright part take
    /// range: 0.0 to 1.0(inclusive)
    /// default: 0.5
    bright_ratio: Option<f64>,
    #[argh(option)]
    /// how color would behave
    /// none: both converted to gray scale by srgb->oklch
    /// bright: use color from bright part, color would leak to dark part
    /// dark: use color from dark part, color would leak to bright part
    /// both: use color from both part, color would leak to the other part
    color_from: Option<ColorFrom>,
    #[argh(switch)]
    /// output preview image of viewed against a white/black background
    /// will output to `out_path` but with extension prefixed with .bright, .dark
    /// e.g. out.bright.png, out.dark.png
    /// would overwrite!
    preview: bool,
}

fn main() -> anyhow::Result<()> {
    let Args {
        bright_path,
        dark_path,
        out_path,
        bright_ratio,
        color_from,
        preview,
    } = argh::from_env();

    let bright_ratio =
        BrightRatio::try_from(bright_ratio.unwrap_or(0.5))?;
    let color_from = color_from.unwrap_or_default();

    let bright = image::open(bright_path)
        .context(r#"failed to open "bright" image"#)?
        .to_rgba8();
    let dark = image::open(dark_path)
        .context(r#"failed to open "dark" image"#)?
        .to_rgba8();

    if bright.dimensions() != dark.dimensions() {
        bail!(r#"size of "bright" and "dark" image doesn't match"#);
    }

    let (width, height) = bright.dimensions();

    let mut out = ImageBuffer::<Rgba<u8>, _>::new(width, height);

    out.par_enumerate_pixels_mut().for_each(|(x, y, out)| {
        let bright = bright.get_pixel(x, y);
        let dark = dark.get_pixel(x, y);

        let bright = px_to_dvec4(*bright).xyz();
        let dark = px_to_dvec4(*dark).xyz();

        let blended = blend_px(bright, dark, color_from, bright_ratio);

        let out_px = dvec4_to_px(blended);
        out.0 = out_px;
    });

    out.save(&out_path).context("failed to save output image")?;

    if preview {
        let mut preview = ImageBuffer::<Rgba<u8>, _>::new(width, height);
        let fill_preview =
            |bg: DVec4, preview: &mut ImageBuffer<Rgba<u8>, _>| {
                preview.par_enumerate_pixels_mut().for_each(
                    |(x, y, preview)| {
                        let px = out.get_pixel(x, y);
                        let px = alpha_blend(bg, px_to_dvec4(*px));
                        preview.0 = dvec4_to_px(px);
                    },
                );
            };
        let preview_path = |name: &'static str| {
            out_path.with_extension(format!(
                "{name}.{}",
                out_path
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
            ))
        };

        fill_preview(DVec4::splat(1.0), &mut preview);
        preview
            .save(preview_path("bright"))
            .context("failed to save bright preview")?;
        fill_preview(DVec3::splat(0.0).extend(1.0), &mut preview);
        preview
            .save(preview_path("dark"))
            .context("failed to save dark preview")?;
    }

    Ok(())
}

fn alpha_blend(bg: DVec4, fg: DVec4) -> DVec4 {
    (fg.xyz() * fg.w + bg.xyz() * (bg.w - fg.w)).extend(bg.w)
}

fn px_to_dvec4(px: Rgba<u8>) -> DVec4 {
    DVec4::from_array(px.0.map(f64::from).map(|it| it / 255.0))
}

fn dvec4_to_px(px: DVec4) -> [u8; 4] {
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    px.to_array()
        .map(|it| it * 255.0)
        .map(|it| it.floor() as u8)
}

fn blend_px(
    bright: DVec3,
    dark: DVec3,
    color_from: ColorFrom,
    bright_ratio: BrightRatio,
) -> DVec4 {
    let dark_ratio = 1.0 - *bright_ratio;
    assert!((0.0..=1.0).contains(&dark_ratio));

    let bright = bright * *bright_ratio + dark_ratio;
    let dark = dark * dark_ratio;

    let bright_lumi = srgb_lumi(bright);
    let dark_lumi = srgb_lumi(dark);

    let alpha_1ch = dark_lumi + 1.0 - bright_lumi;
    let alpha_1ch = if alpha_1ch == 0.0 {
        0.000_000_001
    } else {
        alpha_1ch
    };
    let alpha_3ch = dark + 1.0 - bright;
    let alpha_3ch =
        alpha_3ch.map(|it| if it == 0.0 { 0.000_000_001 } else { it });

    match color_from {
        ColorFrom::None => DVec3::splat(dark_lumi / alpha_1ch),
        ColorFrom::Bright => (bright + alpha_1ch - 1.0) / alpha_1ch,
        ColorFrom::Dark => dark / alpha_1ch,
        ColorFrom::Both => dark / alpha_3ch,
    }
    .extend(alpha_1ch)
}

fn srgb_lumi(color: DVec3) -> f64 {
    #[expect(clippy::cast_possible_truncation)]
    let color =
        OpaqueColor::<Srgb>::new(color.to_array().map(|it| it as f32));
    let oklch = color.convert::<Oklch>();
    f64::from(oklch.components[0])
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter, IntoStaticStr,
)]
enum ColorFrom {
    #[default]
    None,
    Bright,
    Dark,
    Both,
}

impl FromStr for ColorFrom {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::iter()
            .find(|it| Into::<&'static str>::into(it).to_lowercase() == s)
            .ok_or_else(|| {
                anyhow!(
                    "unknown color from {s}, available: {:?}",
                    Self::iter().map(Into::<&'static str>::into)
                )
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct BrightRatio(f64);

impl TryFrom<f64> for BrightRatio {
    type Error = anyhow::Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !(0.0..=1.0).contains(&value) {
            bail!("invalid ratio of bright image");
        }

        Ok(Self(value))
    }
}

impl Deref for BrightRatio {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
