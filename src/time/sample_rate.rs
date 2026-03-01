#[allow(dead_code)]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SampleRate {
    Hz8000 = 0x00,
    Hz11025 = 0x01,
    Hz16000 = 0x02,
    Hz22050 = 0x03,
    Hz44100 = 0x04,
    Hz48000 = 0x05,
    Hz88200 = 0x06,
    Hz96000 = 0x07,
    Hz176400 = 0x08,
    Hz192000 = 0x09,
    Hz352800 = 0x0a,
    Hz384000 = 0x0b,
}

impl TryFrom<usize> for SampleRate {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, ()> {
        match value {
            8000 => Ok(Self::Hz8000),
            11025 => Ok(Self::Hz11025),
            16000 => Ok(Self::Hz16000),
            22050 => Ok(Self::Hz22050),
            44100 => Ok(Self::Hz44100),
            48000 => Ok(Self::Hz48000),
            88200 => Ok(Self::Hz88200),
            96000 => Ok(Self::Hz96000),
            176400 => Ok(Self::Hz176400),
            192000 => Ok(Self::Hz192000),
            352800 => Ok(Self::Hz352800),
            384000 => Ok(Self::Hz384000),
            _ => Err(()),
        }
    }
}

impl Into<u32> for SampleRate {
    fn into(self) -> u32 {
        match self {
            SampleRate::Hz8000 => 8000,
            SampleRate::Hz11025 => 11025,
            SampleRate::Hz16000 => 16000,
            SampleRate::Hz22050 => 22050,
            SampleRate::Hz44100 => 44100,
            SampleRate::Hz48000 => 48000,
            SampleRate::Hz88200 => 88200,
            SampleRate::Hz96000 => 96000,
            SampleRate::Hz176400 => 176400,
            SampleRate::Hz192000 => 192000,
            SampleRate::Hz352800 => 352800,
            SampleRate::Hz384000 => 384000,
        }
    }
}

impl Into<u64> for SampleRate {
    fn into(self) -> u64 {
        Into::<u32>::into(self) as u64
    }
}

impl Into<f32> for SampleRate {
    fn into(self) -> f32 {
        match self {
            SampleRate::Hz8000 => 8000f32,
            SampleRate::Hz11025 => 11025f32,
            SampleRate::Hz16000 => 16000f32,
            SampleRate::Hz22050 => 22050f32,
            SampleRate::Hz44100 => 44100f32,
            SampleRate::Hz48000 => 48000f32,
            SampleRate::Hz88200 => 88200f32,
            SampleRate::Hz96000 => 96000f32,
            SampleRate::Hz176400 => 176400f32,
            SampleRate::Hz192000 => 192000f32,
            SampleRate::Hz352800 => 352800f32,
            SampleRate::Hz384000 => 384000f32,
        }
    }
}

impl Into<f64> for SampleRate {
    fn into(self) -> f64 {
        match self {
            SampleRate::Hz8000 => 8000f64,
            SampleRate::Hz11025 => 11025f64,
            SampleRate::Hz16000 => 16000f64,
            SampleRate::Hz22050 => 22050f64,
            SampleRate::Hz44100 => 44100f64,
            SampleRate::Hz48000 => 48000f64,
            SampleRate::Hz88200 => 88200f64,
            SampleRate::Hz96000 => 96000f64,
            SampleRate::Hz176400 => 176400f64,
            SampleRate::Hz192000 => 192000f64,
            SampleRate::Hz352800 => 352800f64,
            SampleRate::Hz384000 => 384000f64,
        }
    }
}
