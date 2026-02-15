use anyhow::{Context, Error, Result, anyhow};
use coreaudio::audio_unit::AudioFormat;
use coreaudio::audio_unit::audio_format::LinearPcmFlags;
use hound::{SampleFormat, WavSpec};
use objc2_audio_toolbox::{
    AudioComponentDescription, AudioComponentFindNext, AudioComponentInstanceDispose,
    AudioComponentInstanceNew, AudioOutputUnitStart, AudioOutputUnitStop, AudioUnit,
    AudioUnitElement, AudioUnitGetProperty, AudioUnitInitialize, AudioUnitPropertyID,
    AudioUnitScope, AudioUnitSetProperty, AudioUnitUninitialize,
    kAudioOutputUnitProperty_CurrentDevice, kAudioOutputUnitProperty_EnableIO,
    kAudioUnitManufacturer_Apple, kAudioUnitProperty_SampleRate, kAudioUnitProperty_StreamFormat,
    kAudioUnitScope_Global, kAudioUnitScope_Input, kAudioUnitScope_Output,
    kAudioUnitSubType_HALOutput, kAudioUnitType_Output,
};
use objc2_core_audio::{AudioObjectID, kAudioDevicePropertyBufferFrameSize, kAudioHardwareNoError};
use objc2_core_audio_types::AudioStreamBasicDescription;
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::ptr::{NonNull, null};
use std::{mem, ptr};

pub struct CoreAudioUnit {
    instance: AudioUnit,
}

macro_rules! try_status_or_return {
    ($status:expr) => {
        if $status != kAudioHardwareNoError as i32 {
            return Err(Error::msg($status));
        }
    };
}

impl CoreAudioUnit {
    pub fn new_with_flags(flags: u32, mask: u32) -> Result<Self> {
        // A description of the audio unit we desire.
        let desc = AudioComponentDescription {
            componentType: kAudioUnitType_Output,
            componentSubType: kAudioUnitSubType_HALOutput,
            componentManufacturer: kAudioUnitManufacturer_Apple,
            componentFlags: flags,
            componentFlagsMask: mask,
        };

        unsafe {
            // Find the default audio unit for the description.
            //
            // From the "Audio Unit Hosting Guide for iOS":
            //
            // Passing NULL to the first parameter of AudioComponentFindNext tells this function to
            // find the first system audio unit matching the description, using a system-defined
            // ordering. If you instead pass a previously found audio unit reference in this
            // parameter, the function locates the next audio unit matching the description.
            let component = AudioComponentFindNext(ptr::null_mut(), NonNull::from(&desc));
            if component.is_null() {
                return Err(anyhow!("failed to find audio component"));
            }

            // Create an instance of the default audio unit using the component.
            let mut instance_uninit = mem::MaybeUninit::<AudioUnit>::uninit();
            try_status_or_return!(AudioComponentInstanceNew(
                component,
                NonNull::from(&mut instance_uninit).cast()
            ));
            let instance: AudioUnit = instance_uninit.assume_init();

            // Initialize the audio unit!
            try_status_or_return!(AudioUnitInitialize(instance));

            Ok(Self { instance })
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        unsafe {
            try_status_or_return!(AudioUnitInitialize(self.instance));
        }
        Ok(())
    }

    pub fn uninitialize(&mut self) -> Result<()> {
        unsafe {
            try_status_or_return!(AudioUnitUninitialize(self.instance));
        }
        Ok(())
    }

    fn dispose(&mut self) -> Result<()> {
        unsafe {
            try_status_or_return!(AudioComponentInstanceDispose(self.instance));
        }
        Ok(())
    }

    fn set_property<T>(
        &mut self,
        property_id: AudioUnitPropertyID,
        scope: AudioUnitScope,
        elem: AudioUnitElement,
        maybe_data: Option<&T>,
    ) -> Result<()> {
        let (data_ptr, size) = maybe_data
            .map(|data| {
                let ptr = data as *const _ as *const c_void;
                let size = size_of::<T>() as u32;
                (ptr, size)
            })
            .unwrap_or_else(|| (null(), 0));
        unsafe {
            try_status_or_return!(AudioUnitSetProperty(
                self.instance,
                property_id,
                scope,
                elem,
                data_ptr,
                size
            ));
            Ok(())
        }
    }

    fn get_property<T>(&self, id: u32, scope: AudioUnitScope, elem: AudioUnitElement) -> Result<T> {
        let mut size = size_of::<T>() as u32;
        let mut data_uninit = MaybeUninit::<T>::uninit();
        let data_ptr = NonNull::from(&mut data_uninit).cast::<c_void>();
        let size_ptr = NonNull::from(&mut size);

        unsafe {
            try_status_or_return!(AudioUnitGetProperty(
                self.instance,
                id,
                scope,
                elem,
                data_ptr,
                size_ptr
            ));
            let data: T = data_uninit.assume_init();
            Ok(data)
        }
    }

    pub fn enable_io_input(&mut self) -> Result<()> {
        let value = 1u32;
        self.set_property(
            kAudioOutputUnitProperty_EnableIO,
            kAudioUnitScope_Input,
            1,
            Some(&value),
        )
    }

    pub fn disable_io_input(&mut self) -> Result<()> {
        let value = 0u32;
        self.set_property(
            kAudioOutputUnitProperty_EnableIO,
            kAudioUnitScope_Input,
            1,
            Some(&value),
        )
    }

    pub fn enable_io_output(&mut self) -> Result<()> {
        let value = 1u32;
        self.set_property(
            kAudioOutputUnitProperty_EnableIO,
            kAudioUnitScope_Output,
            0,
            Some(&value),
        )
    }

    pub fn disable_io_output(&mut self) -> Result<()> {
        let value = 0u32;
        self.set_property(
            kAudioOutputUnitProperty_EnableIO,
            kAudioUnitScope_Output,
            0,
            Some(&value),
        )
    }

    pub fn set_device(&mut self, device_id: AudioObjectID) -> Result<()> {
        self.set_property(
            kAudioOutputUnitProperty_CurrentDevice,
            kAudioUnitScope_Global,
            0,
            Some(&device_id),
        )
    }

    pub fn start(&mut self) -> Result<()> {
        unsafe {
            try_status_or_return!(AudioOutputUnitStart(self.instance));
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        unsafe {
            try_status_or_return!(AudioOutputUnitStop(self.instance));
        }
        Ok(())
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) -> Result<()> {
        self.set_property(
            kAudioUnitProperty_SampleRate,
            kAudioUnitScope_Input,
            0,
            Some(&sample_rate),
        )
    }

    pub fn get_sample_rate(&self) -> Result<f64> {
        self.get_property(kAudioUnitProperty_SampleRate, kAudioUnitScope_Input, 0)
    }

    pub fn set_io_buffer_size(&mut self, size: u32) -> Result<()> {
        self.set_property(
            kAudioDevicePropertyBufferFrameSize,
            kAudioUnitScope_Input,
            0,
            Some(&size),
        )
        .context("failed to set io buffer size")
    }

    pub fn get_io_buffer_size(&self) -> Result<u32> {
        let data: u32 = self
            .get_property(
                kAudioDevicePropertyBufferFrameSize,
                kAudioUnitScope_Global,
                0,
            )
            .context("failed to get io buffer size")?;
        Ok(data)
    }

    pub fn set_input_stream_format(&mut self, asbd: AudioStreamBasicDescription) -> Result<()> {
        self.set_property(
            kAudioUnitProperty_StreamFormat,
            kAudioUnitScope_Output,
            1,
            Some(&asbd),
        )
    }

    pub fn set_input_stream_format_wav(&mut self, spec: &WavSpec) -> Result<()> {
        let asbd = to_asbd(spec);
        self.set_input_stream_format(asbd)
    }

    pub fn set_output_stream_format(&mut self, asbd: AudioStreamBasicDescription) -> Result<()> {
        self.set_property(
            kAudioUnitProperty_StreamFormat,
            kAudioUnitScope_Input,
            0,
            Some(&asbd),
        )
    }

    pub fn set_output_stream_format_wav(&mut self, spec: &WavSpec) -> Result<()> {
        let asbd = to_asbd(spec);
        self.set_output_stream_format(asbd)
    }

    pub fn get_output_stream_format(&self) -> Result<AudioStreamBasicDescription> {
        self.get_property(kAudioUnitProperty_StreamFormat, kAudioUnitScope_Input, 0)
    }

    pub fn get_input_stream_format(&self) -> Result<AudioStreamBasicDescription> {
        self.get_property(kAudioUnitProperty_StreamFormat, kAudioUnitScope_Output, 1)
    }
}

fn to_asbd(spec: &WavSpec) -> AudioStreamBasicDescription {
    let format_flag = match spec.sample_format {
        SampleFormat::Float => LinearPcmFlags::IS_FLOAT,
        SampleFormat::Int => LinearPcmFlags::IS_SIGNED_INTEGER,
    };

    let flags = format_flag | LinearPcmFlags::IS_PACKED;
    let (format, maybe_flag) = AudioFormat::LinearPCM(flags).as_format_and_flag();

    let flag = maybe_flag.unwrap_or(u32::MAX - 2147483647);

    let bytes_per_sample = spec.bits_per_sample as u32 / 8;
    let bytes_per_frame: u32 = bytes_per_sample * spec.channels as u32;

    let bits_per_channel = spec.bits_per_sample as u32;
    let bytes_per_packet = bytes_per_frame;

    AudioStreamBasicDescription {
        mSampleRate: spec.sample_rate as f64,
        mFormatID: format,
        mFormatFlags: flag,
        mBytesPerPacket: bytes_per_packet,
        mFramesPerPacket: 1,
        mBytesPerFrame: bytes_per_frame,
        mChannelsPerFrame: spec.channels as u32,
        mBitsPerChannel: bits_per_channel,
        mReserved: 0,
    }
}

impl Drop for CoreAudioUnit {
    fn drop(&mut self) {
        self.stop().ok();
        self.uninitialize().ok();
        self.dispose().ok();
    }
}
