use cpal::{Sample as _, SampleFormat};
use zerocopy::{Immutable, IntoBytes, LE, TryFromBytes, U16, U32};

#[derive(Debug, Clone, Copy, TryFromBytes, IntoBytes, Immutable)]
#[repr(C)]
pub struct Header {
    pub channels: U16<LE>,
    pub sample_rate: U32<LE>,
    pub sample_type: SampleType,
}

#[derive(Debug, Clone, Copy, TryFromBytes, IntoBytes, Immutable)]
#[repr(u8)]
pub enum SampleType {
    U8,
    S16LE,
    S32LE,
    F32LE,
}

impl SampleType {
    pub fn size(&self) -> usize {
        match self {
            SampleType::U8 => 1,
            SampleType::S16LE => 2,
            SampleType::S32LE => 4,
            SampleType::F32LE => 4,
        }
    }

    // TODO:
    pub fn read_sample(&self, buf: &[u8]) -> Option<f64> {
        debug_assert_eq!(buf.len(), self.size());

        match self {
            SampleType::U8 => Some(buf[0].to_sample()),
            SampleType::S16LE => {
                let t = *buf.as_array()?;
                Some(i16::from_le_bytes(t).to_sample())
            }
            SampleType::S32LE => {
                let t = *buf.as_array()?;
                Some(i32::from_le_bytes(t).to_sample())
            }
            SampleType::F32LE => {
                let t = *buf.as_array()?;
                Some(f32::from_le_bytes(t).to_sample())
            }
        }
    }
}

// TODO:
impl TryFrom<SampleFormat> for SampleType {
    type Error = SampleFormat;

    fn try_from(value: SampleFormat) -> Result<Self, Self::Error> {
        Ok(match value {
            SampleFormat::U8 => SampleType::U8,
            SampleFormat::I16 => SampleType::S16LE,
            SampleFormat::I32 => SampleType::S32LE,
            SampleFormat::F32 => SampleType::F32LE,
            fmt => return Err(fmt),
        })
    }
}
