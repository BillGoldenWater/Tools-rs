use std::io::{Read, Write, stdin, stdout};

use anyhow::{Context, bail, ensure};
use argh::{FromArgs, from_env};
use pua_codec::Data;

#[derive(FromArgs)]
/// A tool for encode data using Unicode Private Use Area
struct Args {
    /// text mode (default)
    #[argh(switch, short = 't')]
    text: bool,

    /// text mode UTF-16
    #[argh(switch, short = 'T')]
    text_utf16: bool,

    /// binary mode
    #[argh(switch, short = 'b')]
    binary: bool,

    /// decode mode
    #[argh(switch, short = 'd')]
    decode: bool,

    /// output file name, use - for stdout, default is -
    #[argh(option, short = 'o')]
    output: Option<String>,

    /// input file name, use - for stdin, default is -
    #[argh(positional)]
    input: Option<String>,
}

fn main() -> anyhow::Result<()> {
    const IO_BUF_SIZE: usize = 4096;
    const INPUT_BUF_SIZE: usize = 4096;

    let mut args: Args = from_env();
    let modes_count = [args.text, args.text_utf16, args.binary]
        .into_iter()
        .filter(|it| *it)
        .count();
    ensure!(modes_count <= 1, "mode duplicated");

    if modes_count == 0 {
        args.text = true;
    }

    let input = args.input.unwrap_or_else(|| "-".into());
    let output = args.output.unwrap_or_else(|| "-".into());

    let mut input: Box<dyn Read> = {
        if input == "-" {
            Box::new(stdin())
        } else {
            let file = std::fs::OpenOptions::new()
                .read(true)
                .open(input)
                .context("open input")?;
            Box::new(file)
        }
    };
    let mut output: Box<dyn Write> = {
        if output == "-" {
            Box::new(stdout())
        } else {
            let file = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(output)
                .context("open output")?;
            Box::new(file)
        }
    };

    let mut read_text_input =
        |process_input: &mut dyn FnMut(
            &str,
            bool,
        )
            -> anyhow::Result<usize>| {
            let mut read_buf = vec![0u8; IO_BUF_SIZE];
            let mut swap = [0u8; 3];
            let mut swap_n = 0;

            let mut input_buf = String::new();
            let mut input_swap = String::new();

            let mut eof = false;
            'process: loop {
                'read_input: loop {
                    let n = input
                        .read(&mut read_buf[swap_n..])
                        .context("read input")?;

                    if n == 0 {
                        if swap_n != 0 {
                            bail!(
                                "input is not valid utf8, \
                                possibly truncated"
                            );
                        }

                        eof = true;
                        break 'read_input;
                    }

                    let mut chunks = read_buf[..swap_n + n].utf8_chunks();

                    let chunk = chunks
                        .next()
                        .expect("expect non zero data when n != 0");

                    input_buf.push_str(chunk.valid());
                    if !chunk.invalid().is_empty() {
                        if chunks.next().is_some() {
                            bail!("input is not valid utf8");
                        }

                        swap_n = chunk.invalid().len();
                        swap[..swap_n].copy_from_slice(chunk.invalid());
                        read_buf[..swap_n]
                            .copy_from_slice(&swap[..swap_n]);
                    } else {
                        swap_n = 0;
                    }

                    if input_buf.len() >= INPUT_BUF_SIZE {
                        break 'read_input;
                    }
                }

                if eof {
                    break 'process;
                }

                let len = process_input(&input_buf, false)?;

                if len != input_buf.len() {
                    let rest = &input_buf[len..];
                    input_swap.clear();
                    input_swap.push_str(rest);
                    input_buf.clear();
                    input_buf.push_str(&input_swap);
                }
            }

            let len = process_input(&input_buf, true)?;
            assert_eq!(len, input_buf.len());

            Ok(())
        };

    let mut write_output =
        |data: &[u8]| output.write_all(data).context("write output");

    let mut write_data =
        |data: Data<'_>| write_output(data.encode().as_bytes());

    let is_encode = !args.decode;
    if is_encode {
        if args.text {
            read_text_input(&mut |buf, eof| {
                if !eof {
                    for len in (2..=(buf.len() / 2 * 2)).rev().step_by(2)
                    {
                        if buf.is_char_boundary(len) {
                            write_data(Data::Text(&buf[..len]))?;
                            return Ok(len);
                        }
                    }
                    Ok(0)
                } else {
                    write_data(Data::Text(buf))?;

                    Ok(buf.len())
                }
            })
            .context("read text input")?;
        } else if args.text_utf16 {
            read_text_input(&mut |buf, _| {
                write_data(Data::TextUtf16(buf))?;

                Ok(buf.len())
            })
            .context("read text input")?;
        } else if args.binary {
            let mut buf = vec![0u8; IO_BUF_SIZE];
            let mut rest_n = 0;

            loop {
                let n = input
                    .read(&mut buf[rest_n..])
                    .context("read input")?;

                if n == 0 {
                    break;
                }

                let n = rest_n + n;
                if n % 2 == 1 {
                    rest_n = 1;
                    let n = n - 1;
                    if n > 0 {
                        write_data(Data::Binary(&buf[..n]))?;
                    }
                    buf.swap(n, 0);
                } else {
                    write_data(Data::Binary(&buf[..n]))?;
                    rest_n = 0;
                }
            }

            if rest_n > 0 {
                write_data(Data::Binary(&buf[..rest_n]))?;
            }
        } else {
            unreachable!()
        }
    } else if args.text {
        let mut input_buf = Vec::<u8>::with_capacity(IO_BUF_SIZE);
        let mut swap = [0u8; 3];
        let mut swap_n = 0;

        read_text_input(&mut |buf, _| {
            let data =
                Data::decode_binary(buf).context("decode_binary")?;
            input_buf.extend(data);

            if input_buf.len() >= IO_BUF_SIZE {
                let mut chunks = input_buf.utf8_chunks();
                let chunk = chunks.next().expect("expect non zero data");
                print!("{}", chunk.valid());

                if !chunk.invalid().is_empty() {
                    if chunks.next().is_some() {
                        bail!("data is not valid utf8");
                    }

                    swap_n = chunk.invalid().len();
                    swap[..swap_n].copy_from_slice(chunk.invalid());
                    input_buf[..swap_n].copy_from_slice(&swap[..swap_n]);
                } else {
                    swap_n = 0;
                }
            }

            Ok(buf.len())
        })
        .context("read text input")?;

        let data = str::from_utf8(&input_buf)
            .context("data is not valid utf8 (eof)")?;
        print!("{data}");
    } else if args.text_utf16 {
        let mut input_buf = Vec::<u16>::with_capacity(INPUT_BUF_SIZE);
        read_text_input(&mut |buf, _| {
            let data = Data::decode_u16(buf).context("decode_u16")?;
            input_buf.extend(data);

            if input_buf.len() >= INPUT_BUF_SIZE && input_buf.len() > 1 {
                let result = String::from_utf16(&input_buf);
                if let Ok(result) = result {
                    print!("{result}");
                } else {
                    let n = input_buf.len() - 1;
                    let data = String::from_utf16(&input_buf[..n])
                        .context("data is invalid utf16")?;
                    print!("{data}");
                    input_buf.swap(n, 0);
                }
            }

            Ok(buf.len())
        })
        .context("read text input")?;

        let data = String::from_utf16(&input_buf)
            .context("data is invalid utf16 (eof)")?;
        print!("{data}");
    } else if args.binary {
        read_text_input(&mut |buf, _| {
            let data =
                Data::decode_binary(buf).context("decode_binary")?;
            write_output(&data)?;

            Ok(buf.len())
        })
        .context("read text input")?;
    } else {
        unreachable!()
    }

    Ok(())
}
