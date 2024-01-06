use std::path::Path;

use image::{GenericImageView, ImageBuffer, Luma, Pixel};
use itertools::Itertools;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let files = std::fs::read_dir("bad_apple")?
    .map(|it| it.map(|it| it.path()))
    .collect::<Result<Vec<_>, _>>()?;

  std::fs::create_dir_all("out")?;

  for file in files {
    // dbg!(&file);
    convert(
      file.parent().unwrap(),
      file.file_name().unwrap().to_str().unwrap(),
    )?;
  }

  Ok(())
}

fn convert(path: &Path, name: &str) -> Result<(), Box<dyn std::error::Error>> {
  let img = image::open(path.join(name))?;
  let (width, height) = img.dimensions();

  type Target = u16;
  const TARGET_HEIGHT: u32 = 320;

  let mut out_img = ImageBuffer::<Luma<Target>, _>::new(width, TARGET_HEIGHT);
  let height_scale = TARGET_HEIGHT as f64 / height as f64;
  let color_scale = Target::MAX as f64 / TARGET_HEIGHT as f64;

  let mut colors = vec![Vec::<Target>::with_capacity(TARGET_HEIGHT as usize); width as usize];

  for (x, y, color) in img.pixels() {
    let is_white = color.to_luma()[0] > i8::MAX as u8;
    if is_white {
      let y = (y as f64 * height_scale) as u32;

      let luma = (y as f64 * color_scale) as i64;
      let luma = luma as i64 - Target::MAX as i64;
      let luma = luma.abs() as Target;

      colors[x as usize].push(luma);
    }
  }

  for (x, col) in colors.iter().enumerate() {
    for (y, &luma) in col.iter().dedup().sorted().rev().enumerate() {
      out_img.put_pixel(x as u32, y as u32, Luma([luma]));
    }
  }

  out_img.save(format!("out/{name}"))?;

  Ok(())
}
