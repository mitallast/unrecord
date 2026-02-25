use hound::{Error, Result, WavReader, WavSpec, WavWriter};
use log::info;
use std::path::Path;

#[inline]
fn i8_to_f32(x: i8) -> f32 {
    (x as f32) / 128.0
}

#[inline]
fn i16_to_f32(x: i16) -> f32 {
    (x as f32) / 32768.0
}

#[inline]
fn i32_to_f32(x: i32) -> f32 {
    (x as f32) / 2147483648.0 // 2^31
}

#[inline]
fn i32_24bit_to_f32(x: i32) -> f32 {
    (x as f32) / 8_388_608.0 // 2^23
}

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<(f64, Vec<f32>)> {
    let reader = WavReader::open(path)?;
    let spec = reader.spec();
    if spec.channels != 2 {
        return Err(Error::FormatError("expected 2 channels"));
    }
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.into_samples::<f32>().map(|s| s.unwrap()).collect(),
        hound::SampleFormat::Int => match spec.bits_per_sample {
            8 => reader
                .into_samples::<i8>()
                .map(|s| {
                    let sample = s.unwrap();
                    i8_to_f32(sample)
                })
                .collect(),
            16 => reader
                .into_samples::<i16>()
                .map(|s| {
                    let sample = s.unwrap();
                    i16_to_f32(sample)
                })
                .collect(),
            24 => reader
                .into_samples::<i32>()
                .map(|s| {
                    let sample = s.unwrap();
                    i32_24bit_to_f32(sample)
                })
                .collect(),
            32 => reader
                .into_samples::<i32>()
                .map(|s| {
                    let sample = s.unwrap();
                    i32_to_f32(sample)
                })
                .collect(),
            _ => return Err(Error::FormatError("Unsupported sample format")),
        },
    };
    Ok((spec.sample_rate as f64, samples))
}

pub fn write_file<P: AsRef<Path>>(path: P, sample_rate: f64, samples: &Vec<f32>) -> Result<()> {
    let spec = WavSpec {
        channels: 2,
        sample_rate: sample_rate as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    info!("open wav writer");
    let mut writer = WavWriter::create(path, spec)?;
    info!("write samples");
    samples
        .iter()
        .for_each(|s| writer.write_sample(*s).unwrap());
    info!("flush wav file");
    writer.finalize()?;
    Ok(())
}
