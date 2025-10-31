use anyhow::{Context, bail, ensure};

#[derive(Debug, Clone, Copy)]
pub enum Data<'a> {
    Text(&'a str),
    TextUtf16(&'a str),
    Binary(&'a [u8]),
}

impl Data<'_> {
    pub fn encode(self) -> String {
        match self {
            Self::Text(text) => Data::Binary(text.as_bytes()).encode(),
            Self::TextUtf16(text) => {
                text.encode_utf16().map(encode_u16).collect::<String>()
            }
            Self::Binary(data) => {
                if data.is_empty() {
                    return String::new();
                }

                let mut chars =
                    Vec::<char>::with_capacity(data.len().div_ceil(2));

                let mut data = data.chunks_exact(2);
                for data in &mut data {
                    let data = u16::from_be_bytes([data[0], data[1]]);
                    chars.push(encode_u16(data));
                }

                if let [data] = data.remainder() {
                    chars.push(encode_u8(*data));
                }

                String::from_iter(chars)
            }
        }
    }

    pub fn decode_text(data: &str) -> anyhow::Result<String> {
        let data =
            Self::decode_binary(data).context("decode as binary data")?;

        String::from_utf8(data).context("data is invalid utf16")
    }

    pub fn decode_text_utf16(data: &str) -> anyhow::Result<String> {
        let result = Self::decode_u16(data)?;

        String::from_utf16(&result).context("data is invalid utf16")
    }

    pub fn decode_binary(data: &str) -> anyhow::Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(vec![]);
        }

        let count = data.chars().count();
        let mut result = Vec::<u8>::with_capacity(count * 2);

        let mut data = data.chars();
        for ch in (&mut data).take(count - 1) {
            result.extend(decode_u16(ch)?.to_be_bytes());
        }
        let ch = data.next().expect("expect one left");
        if is_u8(ch) {
            result.push(decode_u8(ch)?);
        } else {
            result.extend(decode_u16(ch)?.to_be_bytes());
        }

        Ok(result)
    }

    pub fn decode_u16(data: &str) -> anyhow::Result<Vec<u16>> {
        data.chars()
            .map(decode_u16)
            .collect::<Result<Vec<_>, _>>()
            .context("decode into u16")
    }
}

const fn encode_u16(data: u16) -> char {
    let data = data as u32;
    let is_first_area = data & 0b10 == 0;

    let ch = if is_first_area {
        0x0F_0000_u32 | data & 0x0F_FFFD
    } else {
        0x10_0000_u32 | data & 0x10_FFFD
    };

    char::from_u32(ch).expect("expect valid Unicode")
}

fn decode_u16(ch: char) -> anyhow::Result<u16> {
    let ch = ch as u32;
    ensure!(ch != 0x0F_FFFF);
    ensure!(ch != 0x10_FFFF);

    let data = match ch & 0xFFFF_0000 {
        0x0F_0000 => 0xFFFF & ch,
        0x10_0000 => 0xFFFF & ch | 0b10,
        _ => {
            bail!("expect in Supplementary PUA");
        }
    };

    Ok(data as u16)
}

const fn encode_u8(data: u8) -> char {
    let data = data as u32;
    let ch = 0xE000 | data;
    char::from_u32(ch).expect("expect valid Unicode")
}

fn decode_u8(ch: char) -> anyhow::Result<u8> {
    ensure!(is_u8(ch));

    let ch = ch as u32;
    let data = if ch & 0xFFFF_FF00 == 0xE000 {
        0xFF & ch
    } else {
        bail!("expect in BMP PUA");
    };

    Ok(data as u8)
}

fn is_u8(ch: char) -> bool {
    let ch = ch as u32;
    (0xE000..0xF800).contains(&(ch & 0xFFFF_FF00))
}

#[cfg(test)]
mod tests {
    use super::Data;

    /// Supplementary Private Use Area-A
    #[test]
    fn supplementary_pua_a() {
        let data = &[2, 1];
        let encoded = Data::Binary(data).encode();
        assert_eq!(encoded, "\u{F0201}");
        assert_eq!(Data::decode_binary(&encoded).unwrap(), data);
    }

    /// Supplementary Private Use Area-B
    #[test]
    fn supplementary_pua_b() {
        let data = &[1, 2];
        let encoded = Data::Binary(data).encode();
        assert_eq!(encoded, "\u{100100}");
        assert_eq!(Data::decode_binary(&encoded).unwrap(), data);
    }

    /// Basic Multilingual Plane Private Use Area
    #[test]
    fn bmp_pua() {
        let data = &[3];
        let encoded = Data::Binary(data).encode();
        assert_eq!(encoded, "\u{E003}");
        assert_eq!(Data::decode_binary(&encoded).unwrap(), data);
    }

    /// UTF-8
    #[test]
    fn text() {
        let data = "Hello 你好.";
        let encoded = Data::Text(data).encode();
        assert_eq!(
            encoded,
            "\u{f4865}\u{f6c6c}\u{f6f20}\
\u{fe4bd}\u{fa0e5}\u{fa5bd}\u{e02e}"
        );
        assert_eq!(Data::decode_text(&encoded).unwrap(), data);
    }

    /// UTF-16
    #[test]
    fn text_utf16() {
        let data = "Hello 你好.";
        let encoded = Data::TextUtf16(data).encode();
        assert_eq!(
            encoded,
            "\u{f0048}\u{f0065}\u{f006c}\u{f006c}\u{10006d}\
\u{f0020}\u{f4f60}\u{f597d}\u{10002c}"
        );
        assert_eq!(Data::decode_text_utf16(&encoded).unwrap(), data);
    }
}
