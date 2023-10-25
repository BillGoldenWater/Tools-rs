#![warn(missing_debug_implementations)]

use std::{error::Error, fs::File, io::Read, mem::size_of, path::PathBuf};

use image::RgbaImage;
use tracing::{info, instrument};
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(argh::FromArgs, Debug)]
/// represent binary file as image
struct Args {
  #[argh(switch)]
  /// is decode
  decode: bool,
  #[argh(positional)]
  input: PathBuf,
  #[argh(positional)]
  output: PathBuf,
}

const PIXEL_SIZE: usize = size_of::<u32>();
const HEADER_SIZE: usize = size_of::<u32>();

fn main() -> Result<(), Box<dyn Error>> {
  tracing_subscriber::fmt()
    .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT)
    .init();

  let args: Args = argh::from_env();

  run(args)
}

#[instrument(skip_all)]
fn run(args: Args) -> Result<(), Box<dyn Error>> {
  let input_file = File::options().read(true).open(args.input)?;

  if args.decode {
    decode(input_file, args.output)
  } else {
    encode(input_file, args.output)
  }
}

#[instrument(skip_all)]
fn encode(mut input_file: File, output: PathBuf) -> Result<(), Box<dyn Error>> {
  info!("reading");
  let mut content = Vec::with_capacity(input_file.metadata()?.len() as usize);
  input_file.read_to_end(&mut content)?;
  let file_len = content.len();

  info!("preprocessing");
  let total_pixels = (((content.len() + HEADER_SIZE) as f64) / PIXEL_SIZE as f64).ceil();
  let width = total_pixels.sqrt() as u32;
  let height = ((total_pixels / width as f64).ceil()) as u32;

  content.resize_with((width * height) as usize * PIXEL_SIZE, u8::default);
  content.rotate_right(HEADER_SIZE);
  content[..HEADER_SIZE].copy_from_slice(&(file_len as u32).to_be_bytes());

  info!("encoding");
  let image = RgbaImage::from_raw(width, height, content).expect("expect fit");
  info!("saving");
  image.save(output)?;

  Ok(())
}

#[instrument(skip_all)]
fn decode(mut input_file: File, output: PathBuf) -> Result<(), Box<dyn Error>> {
  info!("reading");
  let mut input_buf = Vec::with_capacity(input_file.metadata()?.len() as usize);
  input_file.read_to_end(&mut input_buf)?;

  info!("decoding");
  let image = image::load_from_memory(&input_buf)?;
  let image = image.to_rgba8();
  let mut img_raw = image.as_raw().clone();

  info!("processing");
  let mut header = [0_u8; HEADER_SIZE];
  header.copy_from_slice(&img_raw[..HEADER_SIZE]);

  let len = u32::from_be_bytes(header) as usize;
  img_raw.rotate_left(HEADER_SIZE);

  info!("saving");
  std::fs::write(
    output,
    &img_raw[..len.min(img_raw.len().saturating_sub(HEADER_SIZE))],
  )?;

  Ok(())
}
