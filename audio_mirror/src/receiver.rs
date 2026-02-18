use std::{
    io::{self, BufReader, Read},
    net::TcpStream,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use audioadapter_buffers::direct::InterleavedSlice;
use cpal::{
    Host, StreamConfig, StreamError,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use ringbuf::traits::{Consumer, Producer, Split};
use rubato::{FixedSync, Resampler};
use zerocopy::TryFromBytes as _;

use crate::{protocol::Header, utils::get_default_output};

pub fn run(addr: &str) {
    let host = cpal::default_host();

    let mut delay = 0.0_f64;
    let mut no_dev_logged = false;
    loop {
        if host.default_output_device().is_none() {
            if !no_dev_logged {
                no_dev_logged = true;
                tracing::info!("no output device available");
            }
            thread::sleep(Duration::from_secs_f64(1.));
            continue;
        } else {
            no_dev_logged = false;
        }

        tracing::info!("connecting {addr}");
        let reason = recv_and_play(&host, addr);
        match reason {
            EndReason::Unknown(error) => {
                tracing::error!("unknown error: {error}");
            }
            EndReason::Network => {
                tracing::warn!("network issue occurred");
            }
            EndReason::Disconnected => {
                tracing::info!("stream disconnected");
                delay = 0.;
            }
            EndReason::OutputChanged => {
                tracing::info!("output change detected, restarting");
                delay = 0.;
            }
        }

        delay = (delay * 2.).clamp(0.1, 10.);
        tracing::info!("retry in {:?}", Duration::from_secs_f64(delay));
        thread::sleep(Duration::from_secs_f64(delay));
    }
}

fn recv_and_play(host: &Host, addr: &str) -> EndReason {
    let (notify_tx, notify_rx) = mpsc::channel::<Notification>();

    let (device, _supported_config, config) =
        match get_default_output(host) {
            Ok(it) => it,
            Err(err) => return EndReason::Unknown(err),
        };

    let rb = ringbuf::SharedRb::new(
        config.sample_rate as usize * config.channels as usize,
    );
    // type name too long for editor type hint to display within line width
    let sp = rb.split();
    let prod = sp.0;
    let mut cons = sp.1;

    let mut last_sample = 0.0;
    let mut underrun_start = Option::<Instant>::None;
    // TODO:
    // assert_eq!(supported_config.sample_format(), SampleFormat::F32);
    let notify_tx_output = notify_tx.clone();
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _info| {
                for sample in data.iter_mut() {
                    if let Some(sample) = cons.try_pop() {
                        last_sample = sample;
                        if let Some(start) = underrun_start.take() {
                            tracing::debug!(
                                "buffer underrun resolved in: {:.2?}",
                                start.elapsed()
                            );
                        }
                    } else if underrun_start.is_none() {
                        tracing::debug!("buffer underrun detected");
                        underrun_start = Some(Instant::now());
                    } else {
                        last_sample /= 2.;
                    }
                    *sample = last_sample as f32
                }
            },
            move |err| {
                notify_tx_output
                    .send(Notification::StreamError(err))
                    .ok();
            },
            None,
        )
        .unwrap();
    stream.play().unwrap();

    // NOTE: spawn receiver after play output stream for minimize buffering
    let res = spawn_receiver(addr, prod, notify_tx, &config);
    if let Err(err) = res {
        tracing::error!("unable to spawn receiver: {err}");

        return EndReason::Network;
    }

    tracing::info!("connected to {addr}");
    while let Ok(notification) = notify_rx.recv() {
        match notification {
            Notification::Tick => {
                let Some(dev) = host.default_output_device() else {
                    return EndReason::OutputChanged;
                };
                let (Ok(cur), Ok(prev)) = (dev.id(), device.id()) else {
                    return EndReason::OutputChanged;
                };
                if cur != prev {
                    return EndReason::OutputChanged;
                }
            }
            Notification::Disconnected => break,
            Notification::StreamError(
                StreamError::DeviceNotAvailable
                | StreamError::StreamInvalidated,
            ) => return EndReason::OutputChanged,
            Notification::StreamError(StreamError::BufferUnderrun) => {
                tracing::warn!("device buffer underrun");
            }
            Notification::StreamError(StreamError::BackendSpecific {
                err,
            }) => {
                return EndReason::Unknown(err.into());
            }
        }
    }

    EndReason::Disconnected
}

enum EndReason {
    // before connected
    Unknown(anyhow::Error),
    Network,

    // after connected
    Disconnected,
    OutputChanged,
}

const NET_SPEED_LOGGING: bool = false;

fn spawn_receiver(
    addr: &str,
    mut prod: impl Producer<Item = f64> + Send + 'static,
    notify_tx: mpsc::Sender<Notification>,
    config: &StreamConfig,
) -> Result<(), RecvError> {
    let mut stream = TcpStream::connect(addr)?;

    let mut header = [0_u8; size_of::<Header>()];
    stream.read_exact(&mut header)?;
    let Ok(header) = Header::try_read_from_bytes(&header) else {
        return Err(RecvError::Protocol("invalid stream header"));
    };

    // TODO:
    assert_eq!(header.channels.get(), config.channels);

    let nb_ch = config.channels as usize;
    let sample_rate = config.sample_rate as usize;

    thread::spawn(move || {
        let mut stream =
            BufReader::with_capacity(2_usize.pow(14), stream);

        let chunk_size_hint = sample_rate / 0.000_5_f64.recip() as usize; // 500μs
        let chunk_size_hint = chunk_size_hint.max(16);

        let mut resampler = rubato::Fft::<f64>::new(
            header.sample_rate.get() as usize,
            sample_rate,
            chunk_size_hint,
            1,
            nb_ch,
            FixedSync::Both,
        )
        .expect("valid construction");

        let mut raw_buf = Vec::<u8>::new();
        let mut input_buf = Vec::<f64>::new();
        let mut output_buf = Vec::<f64>::new();

        let mut count = 0_u128;
        let mut start = Instant::now();
        let mut tick = Instant::now();
        loop {
            if tick.elapsed().as_millis() > 500 {
                if notify_tx.send(Notification::Tick).is_err() {
                    break;
                }
                tick = Instant::now();
            }

            let frame_count = resampler.input_frames_next();
            let sample_count = frame_count * nb_ch;
            raw_buf.resize(sample_count * header.sample_type.size(), 0);

            let res = stream.read_exact(&mut raw_buf);
            match res {
                Ok(()) => {}
                Err(err)
                    if matches!(
                        err.kind(),
                        io::ErrorKind::ConnectionReset
                            | io::ErrorKind::UnexpectedEof
                    ) =>
                {
                    tracing::debug!("disonnected: {err}");
                    break;
                }
                Err(err) => {
                    tracing::error!("receive data, unknown error: {err}");
                }
            };

            if NET_SPEED_LOGGING {
                count += raw_buf.len() as u128;
                if start.elapsed().as_secs_f64() >= 1.0 {
                    let dur = start.elapsed();
                    let speed = count as f64 / dur.as_secs_f64();
                    tracing::info!(
                        "{:.2}Mbps",
                        speed * 8. / 1000. / 1000.,
                    );
                    count = 0;
                    start = Instant::now();
                }
            }

            input_buf.resize(sample_count, 0.0);
            let ty = header.sample_type;
            for (idx, sample) in input_buf.iter_mut().enumerate() {
                let start = idx * ty.size();
                let end = start + ty.size();
                *sample = ty
                    .read_sample(&raw_buf[start..end])
                    .expect("valid size");
            }

            let frame_count_out = resampler.output_frames_next();
            let sample_count_out = frame_count_out * nb_ch;
            output_buf.resize(sample_count_out, 0.0);

            let input =
                InterleavedSlice::new(&input_buf, nb_ch, frame_count)
                    .expect("valid size");
            let mut output = InterleavedSlice::new_mut(
                &mut output_buf,
                nb_ch,
                frame_count_out,
            )
            .expect("valid size");

            let (input_consumed, output_produced) = resampler
                .process_into_buffer(&input, &mut output, None)
                .expect("valid size and channel count");
            assert_eq!(input_consumed, frame_count);
            assert_eq!(output_produced, frame_count_out);

            let n = prod.push_slice(&output_buf);
            if n != output_buf.len() {
                // output should not stall
                break;
            }
        }

        notify_tx.send(Notification::Disconnected).ok();
    });

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum RecvError {
    #[error("network issue: {0}")]
    Network(#[from] io::Error),
    #[error("invalid protocol: {0}")]
    Protocol(&'static str),
}

enum Notification {
    Tick,
    Disconnected,
    StreamError(StreamError),
}
