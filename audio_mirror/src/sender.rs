use std::{
    io::{self, BufWriter, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    thread,
};

use cpal::{
    Device, I24, SampleFormat, SizedSample, StreamConfig,
    SupportedStreamConfig, U24,
    traits::{DeviceTrait, StreamTrait},
};
use dasp_sample::ToSample;
use ringbuf::{
    storage::Heap,
    traits::{Consumer, Producer as _, Split as _},
};
use zerocopy::IntoBytes;

use crate::{
    protocol::{Header, SampleType},
    utils::get_default_output,
};

pub fn run(addr: &str) {
    // TODO: auto device switch
    // TODO: input device
    let host = if cfg!(target_os = "windows") {
        let id = cpal::available_hosts()
            .into_iter()
            .find(|it| it.name() == "Wasapi");
        let host = id.map(|it| {
            cpal::host_from_id(it).expect("from cpal::available_hosts()")
        });
        host.unwrap_or_else(cpal::default_host)
    } else {
        cpal::default_host()
    };

    let listener = TcpListener::bind(addr).unwrap();
    tracing::info!("listening {}", listener.local_addr().unwrap());

    while let Ok((net_stream, addr)) = listener.accept() {
        tracing::info!("accpet: {addr}");

        let (device, supported_config, config) =
            get_default_output(&host).unwrap();
        tracing::info!("config: {supported_config:?}");

        struct P<'p> {
            device: &'p Device,
            supported_config: &'p SupportedStreamConfig,
            config: &'p StreamConfig,
            net_stream: TcpStream,
        }
        let p = P {
            device: &device,
            supported_config: &supported_config,
            config: &config,
            net_stream,
        };
        fn handle<S: GenericSample + Send + 'static>(p: P) {
            let P {
                device,
                supported_config,
                config,
                net_stream,
            } = p;
            handle_stream::<S>(
                device,
                supported_config,
                config,
                net_stream,
            );
        }

        match supported_config.sample_format() {
            SampleFormat::I8 => handle::<i8>(p),
            SampleFormat::I16 => handle::<i16>(p),
            SampleFormat::I24 => handle::<I24>(p),
            SampleFormat::I32 => handle::<i32>(p),
            SampleFormat::I64 => handle::<i64>(p),
            SampleFormat::U8 => handle::<u8>(p),
            SampleFormat::U16 => handle::<u16>(p),
            SampleFormat::U24 => handle::<U24>(p),
            SampleFormat::U32 => handle::<u32>(p),
            SampleFormat::U64 => handle::<u64>(p),
            SampleFormat::F32 => handle::<f32>(p),
            SampleFormat::F64 => handle::<f64>(p),
            fmt => unimplemented!("unsupported {fmt}"),
        }
        tracing::info!("end");
    }
}

fn handle_stream<S: GenericSample + Send + 'static>(
    device: &Device,
    supported_config: &SupportedStreamConfig,
    config: &StreamConfig,
    net_stream: TcpStream,
) {
    let rb = ringbuf::SharedRb::<Heap<S>>::new(
        config.sample_rate as usize * config.channels as usize,
    );
    // type name too long for editor type hint to display within line width
    let sp = rb.split();
    let mut prod = sp.0;
    let cons = sp.1;

    let stop_rx =
        spawn_sender(net_stream, cons, supported_config).unwrap();

    let mut buffer_full = false;

    let stream = device
        .build_input_stream(
            config,
            move |data: &[S], _info| {
                for sample in data {
                    let res = prod.try_push(*sample);
                    if res.is_err() {
                        if !buffer_full {
                            tracing::debug!("buffer full");
                            buffer_full = true;
                        }
                    } else if buffer_full {
                        tracing::debug!("buffer full resolved");
                        buffer_full = false;
                    }
                }
            },
            |err| {
                tracing::error!("stream error: {err:?}");
            },
            None,
        )
        .unwrap();

    stream.play().unwrap();

    stop_rx.recv().ok();
}

fn spawn_sender<S: GenericSample>(
    mut stream: TcpStream,
    mut cons: impl Consumer<Item = S> + Send + 'static,
    config: &SupportedStreamConfig,
) -> anyhow::Result<mpsc::Receiver<()>> {
    let (stop_tx, stop_rx) = mpsc::channel();

    let header = Header {
        channels: config.channels().into(),
        sample_rate: config.sample_rate().into(),
        sample_type: SampleType::try_from(config.sample_format())
            .unwrap(),
    };

    stream.write_all(header.as_bytes()).unwrap();

    thread::spawn(move || {
        let mut stream = BufWriter::new(stream);
        'send: loop {
            for sample in cons.pop_iter() {
                let res = to_le_bytes(sample, &mut stream);
                if res.is_err() {
                    break 'send;
                }
            }

            let res = stream.flush();
            if res.is_err() {
                break;
            }
        }
        stop_tx.send(()).ok();
    });

    Ok(stop_rx)
}

fn to_le_bytes<S: GenericSample>(
    s: S,
    out: &mut impl Write,
) -> io::Result<()> {
    match S::FORMAT {
        SampleFormat::I8 => out.write_all(&[s.to_sample::<i8>() as u8]),
        SampleFormat::I16 => {
            out.write_all(&s.to_sample::<i16>().to_le_bytes())
        }
        SampleFormat::I24 => {
            out.write_all(&s.to_sample::<I24>().inner().to_le_bytes())
        }
        SampleFormat::I32 => {
            out.write_all(&s.to_sample::<i32>().to_le_bytes())
        }
        SampleFormat::I64 => {
            out.write_all(&s.to_sample::<i64>().to_le_bytes())
        }
        SampleFormat::U8 => out.write_all(&[s.to_sample::<u8>()]),
        SampleFormat::U16 => {
            out.write_all(&s.to_sample::<u16>().to_le_bytes())
        }
        SampleFormat::U24 => {
            out.write_all(&s.to_sample::<U24>().inner().to_le_bytes())
        }
        SampleFormat::U32 => {
            out.write_all(&s.to_sample::<u32>().to_le_bytes())
        }
        SampleFormat::U64 => {
            out.write_all(&s.to_sample::<u64>().to_le_bytes())
        }
        SampleFormat::F32 => {
            out.write_all(&s.to_sample::<f32>().to_le_bytes())
        }
        SampleFormat::F64 => {
            out.write_all(&s.to_sample::<f64>().to_le_bytes())
        }
        fmt => unimplemented!("unsupported {fmt}"),
    }
}

macro_rules! trait_generic_sample {
    ($($ty:ty),* $(,)?) => {
        trait GenericSample: SizedSample $(+ ToSample<$ty>)* {}

        impl<T> GenericSample for T where T: SizedSample $(+ ToSample<$ty>)* { }
    };
}

trait_generic_sample!(
    i8, i16, I24, i32, i64, u8, u16, U24, u32, u64, f32, f64
);
