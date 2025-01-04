use core::f64;
use std::{
    mem::size_of,
    path::{Path, PathBuf},
    sync::{
        atomic::{self, AtomicU64},
        Mutex, TryLockError,
    },
    time::Instant,
};

use image::RgbaImage;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn main() -> anyhow::Result<()> {
    // audio settings
    let audio_path = Path::new("audio.bin");
    let sample_rate: usize = 44100;
    type SampleType = f64;
    let sample_size = size_of::<SampleType>();
    let bytes_to_sample = SampleType::from_le_bytes;
    // video settings
    let frame_rate: usize = 60;
    let frame_number_to_output_file_path = |frame_number: u64| {
        PathBuf::from(format!("./output/{frame_number:0>10}.png"))
    };
    // generation settings
    let left_dur: f64 = 0.25;
    let right_dur: f64 = 58.0 / 60.0;
    // ipad
    let width: u32 = 1680;
    let height: u32 = 140;
    // pc
    //let width: u32 = 1625;
    //let height: u32 = 120;
    // other settings
    let log_interval_secs: f64 = 0.2;

    // calculated settings
    let cursor_x = (width as f64 * (left_dur / (left_dur + right_dur)))
        .floor() as u32;
    let total_dur = left_dur + right_dur;
    let rms_chunk_dur = total_dur / width as f64;

    println!("read audio");
    let audio = std::fs::read(audio_path).unwrap();
    let samples = audio
        .chunks_exact(sample_size)
        .map(|sample| bytes_to_sample(sample.try_into().unwrap()))
        .collect_vec();
    let volumes = samples
        .iter()
        .enumerate()
        .map(|(idx, sample)| {
            if idx == 0 {
                0.0
            } else {
                (sample - samples[idx - 1]).abs()
            }
        })
        .collect_vec();

    let secs_to_sample_idx =
        |secs: f64| (secs * (sample_rate as f64)).floor() as usize;
    let get_volume_rms_at = |secs: f64, dur_secs: f64| {
        let start = secs_to_sample_idx(secs);
        let end = secs_to_sample_idx(secs + dur_secs);
        let range = start..end;
        if range.is_empty() {
            volumes[start]
        } else {
            range
                .clone()
                .map(|idx| volumes.get(idx).copied().unwrap_or(0.0))
                .sum::<f64>()
                / range.len() as f64
        }
    };

    let get_volume_rms_data_for_cursor = |cursor_secs: f64| {
        let start = cursor_secs - left_dur;
        (0..width)
            .map(|x| {
                let offset = x as f64 / width as f64 * total_dur;
                get_volume_rms_at(start + offset, rms_chunk_dur)
            })
            .collect_vec()
    };

    let total_frame_count = (samples.len() as f64 / sample_rate as f64
        * frame_rate as f64) as u64;

    let finished_count = AtomicU64::new(0);
    let last_log = Mutex::new((Instant::now(), 0));
    (1..=total_frame_count).into_par_iter().for_each(|frame_num| {
        let timestamp = frame_num as f64 / frame_rate as f64;

        let volume_rms_data = get_volume_rms_data_for_cursor(timestamp);
        let max =
            volume_rms_data.iter().copied().reduce(f64::max).unwrap();
        let scaler = max.recip();
        let scaler = if scaler.is_infinite() { 1.0 } else { scaler };
        let volume_rms_data = volume_rms_data
            .into_iter()
            .map(|it| it * scaler)
            .collect_vec();

        let mut img = RgbaImage::new(width, height);
        for (x, y, px) in img.enumerate_pixels_mut() {
            if x == cursor_x {
                px.0 = [u8::MAX, 0, 0, u8::MAX];
            } else {
                let volume = volume_rms_data[x as usize];
                let threshold = 1.0 - (y as f64 / height as f64);
                let luma = (volume > threshold) as u8 * u8::MAX;
                px.0 = [luma, luma, luma, u8::MAX];
            }
        }
        let out_path = frame_number_to_output_file_path(frame_num);
        img.save(out_path).expect("failed to save output file");

        let count = finished_count.fetch_add(1, atomic::Ordering::Relaxed) + 1;
        match last_log.try_lock() {
            Ok(mut last_log) => {
                let elapsed_sec = last_log.0.elapsed().as_secs_f64();
                if elapsed_sec > log_interval_secs {
                    println!(
                        "{count: >10}/{total_frame_count: >10}, {:.2}%, fps: {:.2}",
                        count as f64 / total_frame_count as f64 * 100.0,
                        (count - last_log.1) as f64 / elapsed_sec,
                    );
                    *last_log = (Instant::now(), count);
                }
            },
            Err(TryLockError::WouldBlock) => {},
            Err(err @ TryLockError::Poisoned(_)) => {
                panic!("poisoned: {err:?}");
            },
        }
    });

    Ok(())
}
