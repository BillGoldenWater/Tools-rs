use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use image::imageops::FilterType;
use image::{GenericImageView, ImageBuffer, Rgb};
use rayon::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let files = std::fs::read_dir("in")?
    .map(|it| it.map(|it| it.path()))
    .collect::<Result<Vec<_>, _>>()?;

  std::fs::create_dir_all("out")?;

  let count = AtomicUsize::new(0);
  files.par_iter().try_for_each(|file| {
    let count = count.fetch_add(1, Ordering::SeqCst);
    println!("{:.2}", count as f64 / files.len() as f64 * 100.0);
    convert(
      file.parent().unwrap(),
      file.file_name().unwrap().to_str().unwrap(),
    )
  })?;

  Ok(())
}

fn convert(path: &Path, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let img = image::open(path.join(name))?;
  let (width, _) = img.dimensions();

  type Target = u8;

  let waveform_height = 1080;
  let px_multi = 8_u32;

  let target_height = waveform_height * px_multi;

  let img = img.resize_exact(width, waveform_height, FilterType::Nearest);

  let y_scale = Target::MAX as f64 / waveform_height as f64;
  let px_scale = px_multi as f64 / u8::MAX as f64;

  let mut colors =
    vec![vec![Vec::<Target>::with_capacity(target_height as usize); width as usize]; 3];

  for (x, y, color) in img.pixels() {
    for (ch, &value) in color.0.iter().take(3).enumerate() {
      let y = (y as f64 * y_scale) as Target;
      let y = y as i64 - Target::MAX as i64;
      let y = y.unsigned_abs() as Target;

      for _ in 0..(value as f64 * px_scale) as usize {
        colors[ch][x as usize].push(y);
      }
    }
  }

  let mut out_img = ImageBuffer::<Rgb<Target>, _>::new(width, target_height);
  for (ch, colors) in colors.iter().enumerate() {
    for (x, col) in colors.iter().enumerate() {
      for (y, &value) in col.iter().enumerate() {
        let px = out_img.get_pixel_mut(x as u32, y as u32);
        px.0[ch] = value;
      }
    }
  }

  out_img.save(format!("out/{name}"))?;

  Ok(())
}
