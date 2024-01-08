use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use image::buffer::ConvertBuffer;
use image::imageops::FilterType;
use image::{GenericImageView, ImageBuffer, Rgb};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

#[derive(argh::FromArgs)]
/// convert image(s) to display in waveform
struct Args {
  /// path to dir that contains all input images
  #[argh(option, short = 'i')]
  pub input: Option<PathBuf>,
  /// path to dir that will output to
  #[argh(option, short = 'o')]
  pub output: Option<PathBuf>,
  /// thread num
  #[argh(option, short = 't')]
  pub thread_num: Option<usize>,
  /// the target waveform height
  #[argh(option, short = 'h')]
  pub waveform_height: Option<u32>,
  /// the brightness level of a pixel
  #[argh(option, short = 'p')]
  pub px_multi: Option<u32>,
  /// enable 16bit output
  #[argh(switch)]
  pub use_16bit: bool,
}

static DEFAULT_IN: OnceLock<PathBuf> = OnceLock::new();
static DEFAULT_OUT: OnceLock<PathBuf> = OnceLock::new();

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  DEFAULT_IN.get_or_init(|| PathBuf::from("in"));
  DEFAULT_OUT.get_or_init(|| PathBuf::from("out"));

  run(&argh::from_env())
}

fn run(args: &Args) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let files = std::fs::read_dir(args.input.as_ref().unwrap_or(DEFAULT_IN.get().unwrap()))?
    .map(|it| it.map(|it| it.path()))
    .collect::<Result<Vec<_>, _>>()?;

  std::fs::create_dir_all(args.output.as_ref().unwrap_or(DEFAULT_OUT.get().unwrap()))?;

  if let Some(thread_num) = args.thread_num.as_ref().copied() {
    rayon::ThreadPoolBuilder::new()
      .num_threads(thread_num.max(1))
      .build_global()?;
  }

  let bar = ProgressBar::new(files.len() as u64).with_style(ProgressStyle::with_template(
    "[{elapsed}] {bar} {percent}% [{pos}/{len}] eta: {eta} {per_sec}",
  )?);

  files.par_iter().try_for_each(|file| {
    let ret = convert(
      args,
      file.parent().unwrap(),
      file.file_name().unwrap().to_str().unwrap(),
    );

    bar.inc(1);

    ret
  })?;

  bar.finish();
  println!("done");

  Ok(())
}

fn convert(
  args: &Args,
  path: &Path,
  name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let img = image::open(path.join(name))?;
  let (width, _) = img.dimensions();

  type Target = u16;

  let waveform_height = args.waveform_height.unwrap_or(1024);
  let px_multi = args.px_multi.unwrap_or(2);

  let target_height = waveform_height * px_multi;

  let img = img.resize_exact(width, waveform_height, FilterType::Nearest);

  let y_scale = Target::MAX as f64 / waveform_height as f64;
  let px_scale = px_multi as f64 / u8::MAX as f64;

  let mut out_img = ImageBuffer::<Rgb<Target>, _>::new(width, target_height);
  let mut y_count = vec![vec![0; width as usize]; 3];

  for (x, y, color) in img.pixels() {
    for (ch, &value) in color.0.iter().take(3).enumerate() {
      let y = (y as f64 * y_scale) as Target;
      let y = y as i64 - Target::MAX as i64;
      let y = y.unsigned_abs() as Target;

      let count = &mut y_count[ch][x as usize];

      for _ in 0..(value as f64 * px_scale) as usize {
        out_img.get_pixel_mut(x, *count).0[ch] = y;
        *count += 1;
      }
    }
  }

  let out = args
    .output
    .as_ref()
    .unwrap_or(DEFAULT_OUT.get().unwrap())
    .join(name);

  if args.use_16bit {
    out_img.save(out)?;
  } else {
    let out_img: ImageBuffer<Rgb<u8>, _> = out_img.convert();
    out_img.save(out)?;
  }

  Ok(())
}
