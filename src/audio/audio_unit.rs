use anyhow::{Context, Result, anyhow};
use coreaudio::audio_unit::audio_format::LinearPcmFlags;
use coreaudio::audio_unit::{AudioUnit, Element, SampleFormat, Scope, StreamFormat};
use coreaudio_sys::kAudioUnitProperty_StreamFormat;
use hound::WavSpec;
use log::info;
use objc2_audio_toolbox::{
    kAudioOutputUnitProperty_CurrentDevice, kAudioOutputUnitProperty_EnableIO,
    kAudioUnitProperty_SampleRate,
};
use objc2_core_audio::{AudioObjectID, kAudioDevicePropertyBufferFrameSize};
use objc2_core_audio_types::AudioStreamBasicDescription;

pub trait CoreAudioUnit {
    fn set_device(&mut self, device_id: AudioObjectID) -> Result<()>;

    fn enable_io_input(&mut self) -> Result<()>;
    fn disable_io_input(&mut self) -> Result<()>;
    fn enable_io_output(&mut self) -> Result<()>;
    fn disable_io_output(&mut self) -> Result<()>;

    fn set_sample_rate(&mut self, sample_rate: f64) -> Result<()>;
    fn get_sample_rate(&self) -> Result<f64>;

    fn set_io_buffer_size(&mut self, size: u32) -> Result<()>;
    fn get_io_buffer_size(&self) -> Result<u32>;

    fn set_input_stream_format_spec(&mut self, spec: &StreamFormat) -> Result<()>;
    fn set_output_stream_format_spec(&mut self, spec: &StreamFormat) -> Result<()>;

    fn set_input_stream_format_wav(&mut self, spec: &WavSpec) -> Result<()>;
    fn set_output_stream_format_wav(&mut self, spec: &WavSpec) -> Result<()>;
}

fn to_asbd(spec: &WavSpec) -> Result<AudioStreamBasicDescription> {
    let format_flag = match spec.sample_format {
        hound::SampleFormat::Float => LinearPcmFlags::IS_FLOAT,
        hound::SampleFormat::Int => LinearPcmFlags::IS_SIGNED_INTEGER,
    };
    let sample_format = match spec.sample_format {
        hound::SampleFormat::Float => SampleFormat::F32,
        hound::SampleFormat::Int => match spec.bits_per_sample {
            8 => SampleFormat::I8,
            16 => SampleFormat::I16,
            24 => SampleFormat::I24,
            32 => SampleFormat::I32,
            _ => return Err(anyhow!("Unsupported sample format")),
        },
    };

    let stream_format = StreamFormat {
        sample_rate: spec.sample_rate as f64,
        sample_format,
        flags: format_flag | LinearPcmFlags::IS_PACKED | LinearPcmFlags::IS_NON_INTERLEAVED,
        channels: spec.channels as u32,
    };

    info!("stream format: {:#?}", stream_format);

    let asbd = stream_format.to_asbd();

    info!("asbd: {:#?}", asbd);

    Ok(asbd)
}

// input=StreamFormat {
//     sample_rate: 44100.0,
//     sample_format: F32,
//     flags: LinearPcmFlags(
//         IS_FLOAT | IS_PACKED | IS_NON_INTERLEAVED,
//     ),
//     channels: 1,
// }
// output=StreamFormat {
//     sample_rate: 44100.0,
//     sample_format: F32,
//     flags: LinearPcmFlags(
//         IS_FLOAT | IS_PACKED | IS_NON_INTERLEAVED,
//     ),
//     channels: 2,
// }
// input_asbd=AudioStreamBasicDescription {
//     mSampleRate: 44100.0,
//     mFormatID: 1819304813,
//     mFormatFlags: 41,
//     mBytesPerPacket: 4,
//     mFramesPerPacket: 1,
//     mBytesPerFrame: 4,
//     mChannelsPerFrame: 1,
//     mBitsPerChannel: 32,
//     mReserved: 0,
// }
// output_asbd=AudioStreamBasicDescription {
//     mSampleRate: 44100.0,
//     mFormatID: 1819304813,
//     mFormatFlags: 41,
//     mBytesPerPacket: 4,
//     mFramesPerPacket: 1,
//     mBytesPerFrame: 4,
//     mChannelsPerFrame: 2,
//     mBitsPerChannel: 32,
//     mReserved: 0,
// }

impl CoreAudioUnit for AudioUnit {
    fn set_device(&mut self, device_id: AudioObjectID) -> Result<()> {
        self.set_property(
            kAudioOutputUnitProperty_CurrentDevice,
            Scope::Global,
            Element::Output,
            Some(&device_id),
        )
        .context("Failed to set current device")
    }

    fn enable_io_input(&mut self) -> Result<()> {
        let value = 1u32;
        self.set_property(
            kAudioOutputUnitProperty_EnableIO,
            Scope::Input,
            Element::Input,
            Some(&value),
        )
        .context("Failed to enable IO input")
    }

    fn disable_io_input(&mut self) -> Result<()> {
        let value = 0u32;
        self.set_property(
            kAudioOutputUnitProperty_EnableIO,
            Scope::Input,
            Element::Input,
            Some(&value),
        )
        .context("Failed to disable IO input")
    }

    fn enable_io_output(&mut self) -> Result<()> {
        let value = 1u32;
        self.set_property(
            kAudioOutputUnitProperty_EnableIO,
            Scope::Output,
            Element::Output,
            Some(&value),
        )
        .context("Failed to enable IO output")
    }

    fn disable_io_output(&mut self) -> Result<()> {
        let value = 0u32;
        self.set_property(
            kAudioOutputUnitProperty_EnableIO,
            Scope::Output,
            Element::Output,
            Some(&value),
        )
        .context("Failed to disable IO output")
    }

    fn set_sample_rate(&mut self, sample_rate: f64) -> Result<()> {
        self.set_property(
            kAudioUnitProperty_SampleRate,
            Scope::Input,
            Element::Output,
            Some(&sample_rate),
        )
        .context("Failed to set sample rate")
    }

    /// Get the **AudioUnit**'s sample rate.
    fn get_sample_rate(&self) -> Result<f64> {
        self.get_property(kAudioUnitProperty_SampleRate, Scope::Input, Element::Output)
            .context("Failed to get sample rate")
    }

    fn set_io_buffer_size(&mut self, size: u32) -> Result<()> {
        self.set_property(
            kAudioDevicePropertyBufferFrameSize,
            Scope::Input,
            Element::Output,
            Some(&size),
        )
        .context("failed to set io buffer size")
    }

    fn get_io_buffer_size(&self) -> Result<u32> {
        let data: u32 = self
            .get_property(
                kAudioDevicePropertyBufferFrameSize,
                Scope::Input,
                Element::Output,
            )
            .context("failed to get io buffer size")?;
        Ok(data)
    }

    fn set_input_stream_format_spec(&mut self, spec: &StreamFormat) -> Result<()> {
        let asbd = spec.to_asbd();
        info!("input asbd: {:#?}", asbd);
        self.set_property(
            kAudioUnitProperty_StreamFormat,
            Scope::Output,
            Element::Input,
            Some(&asbd),
        )
        .context("Failed to set input stream format")
    }

    fn set_output_stream_format_spec(&mut self, spec: &StreamFormat) -> Result<()> {
        let asbd = spec.to_asbd();
        info!("output asbd: {:#?}", asbd);
        self.set_property(
            kAudioUnitProperty_StreamFormat,
            Scope::Input,
            Element::Output,
            Some(&asbd),
        )
        .context("Failed to set output stream format")
    }

    fn set_input_stream_format_wav(&mut self, spec: &WavSpec) -> Result<()> {
        let asbd = to_asbd(spec);
        self.set_property(
            kAudioUnitProperty_StreamFormat,
            Scope::Output,
            Element::Input,
            Some(&asbd),
        )
        .context("Failed to set input stream format")
    }

    fn set_output_stream_format_wav(&mut self, spec: &WavSpec) -> Result<()> {
        let asbd = to_asbd(spec);
        self.set_property(
            kAudioUnitProperty_StreamFormat,
            Scope::Input,
            Element::Output,
            Some(&asbd),
        )
        .context("Failed to set output stream format")
    }
}
