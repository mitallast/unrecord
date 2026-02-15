use anyhow::{Context, Error, Result};
use coreaudio_sys::kAudioObjectPropertyScopeInput;
use objc2_core_audio::{
    AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize, AudioObjectHasProperty,
    AudioObjectID, AudioObjectPropertyAddress, kAudioDevicePropertyAvailableNominalSampleRates,
    kAudioDevicePropertyBufferFrameSize, kAudioDevicePropertyBufferFrameSizeRange,
    kAudioDevicePropertyDeviceManufacturerCFString, kAudioDevicePropertyDeviceNameCFString,
    kAudioDevicePropertyDeviceUID, kAudioDevicePropertyNominalSampleRate,
    kAudioDevicePropertyStreamConfiguration, kAudioHardwareNoError,
    kAudioObjectPropertyElementMain, kAudioObjectPropertyElementWildcard,
    kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyScopeOutput,
};
use objc2_core_audio_types::{AudioBufferList, AudioValueRange};
use objc2_core_foundation::{CFRetained, CFString};
use std::mem::MaybeUninit;
use std::ops::RangeInclusive;
use std::ptr::{NonNull, null};

pub struct CoreAudioDevice {
    device_id: AudioObjectID,
}

macro_rules! try_status_or_return {
    ($status:expr) => {
        if $status != kAudioHardwareNoError as i32 {
            return Err(Error::msg($status));
        }
    };
}

impl CoreAudioDevice {
    pub fn from_id(device_id: AudioObjectID) -> Self {
        Self { device_id }
    }

    pub fn get_id(&self) -> AudioObjectID {
        self.device_id
    }

    fn has_property(&self, property_address: AudioObjectPropertyAddress) -> bool {
        unsafe { AudioObjectHasProperty(self.device_id, NonNull::from(&property_address)) }
    }

    fn get_property_data_size(&self, property_address: AudioObjectPropertyAddress) -> Result<u32> {
        let mut data_size: u32 = 0;
        unsafe {
            let status = AudioObjectGetPropertyDataSize(
                self.device_id,
                NonNull::from(&property_address),
                0,
                null(),
                NonNull::from_mut(&mut data_size),
            );
            try_status_or_return!(status);
            Ok(data_size)
        }
    }

    fn get_property_data<T: Sized>(
        &self,
        property_address: AudioObjectPropertyAddress,
    ) -> Result<T> {
        let data_size = self.get_property_data_size(property_address)?;
        let mut data_uninit = MaybeUninit::<T>::uninit();

        unsafe {
            let status = AudioObjectGetPropertyData(
                self.device_id,
                NonNull::from(&property_address),
                0,
                null(),
                NonNull::from(&data_size),
                NonNull::from(&mut data_uninit).cast(),
            );
            try_status_or_return!(status);
            let data = data_uninit.assume_init();
            Ok(data)
        }
    }

    fn get_property_array<T: Sized>(
        &self,
        property_address: AudioObjectPropertyAddress,
    ) -> Result<Vec<T>> {
        let data_size = self.get_property_data_size(property_address)?;
        let items_count = data_size as usize / size_of::<T>();
        let mut items: Vec<T> = Vec::with_capacity(items_count);

        unsafe {
            let status = AudioObjectGetPropertyData(
                self.device_id,
                NonNull::from(&property_address),
                0,
                null(),
                NonNull::from(&data_size),
                NonNull::new(items.as_mut_ptr()).unwrap().cast(),
            );
            try_status_or_return!(status);
            items.set_len(items_count);
            Ok(items)
        }
    }

    fn get_property_string(&self, property_address: AudioObjectPropertyAddress) -> Result<String> {
        let property_value: *const CFString = self
            .get_property_data(property_address)
            .context("test get cf string")?;

        let property_value = NonNull::new(property_value as *mut CFString)
            .ok_or(Error::msg("property value is null"))?;

        unsafe {
            let property_value = CFRetained::from_raw(property_value);
            Ok(property_value.to_string())
        }
    }

    pub fn get_name(&self) -> Result<String> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyDeviceNameCFString,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };
        self.get_property_string(property_address)
            .context("failed to get device name")
    }

    pub fn get_manufacturer(&self) -> Result<String> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyDeviceManufacturerCFString,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };
        self.get_property_string(property_address)
            .context("failed to get device manufacturer")
    }

    pub fn get_uid(&self) -> Result<String> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyDeviceUID,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };
        self.get_property_string(property_address)
            .context("failed to get device uid")
    }

    pub fn get_io_buffer_size(&self) -> Result<u32> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyBufferFrameSize,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };
        let data: u32 = self
            .get_property_data(property_address)
            .context("failed to get io buffer size")?;
        Ok(data)
    }

    pub fn get_io_buffer_size_range(&self) -> Result<RangeInclusive<u32>> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyBufferFrameSizeRange,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        let range: AudioValueRange = self
            .get_property_data(property_address)
            .context("failed to get device buffer size range")?;

        Ok(RangeInclusive::new(
            range.mMinimum as u32,
            range.mMaximum as u32,
        ))
    }

    pub fn get_sample_rate(&self) -> Result<f64> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyNominalSampleRate,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        let sample_rate: f64 = self
            .get_property_data(property_address)
            .context("failed to get nominal sample rate")?;
        Ok(sample_rate)
    }

    pub fn get_sample_rate_range(&self) -> Result<RangeInclusive<u32>> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyAvailableNominalSampleRates,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };
        let ranges = self
            .get_property_array::<AudioValueRange>(property_address)
            .context("failed to get nominal sample rates")?;

        let minimum = ranges
            .iter()
            .map(|v| v.mMinimum as u32)
            .min()
            .expect("the list must not be empty");

        let maximum = ranges
            .iter()
            .map(|v| v.mMaximum as u32)
            .max()
            .expect("the list must not be empty");

        Ok(RangeInclusive::new(minimum, maximum))
    }

    fn get_channels(&self, property_address: AudioObjectPropertyAddress) -> Result<u32> {
        let buffers = self
            .get_property_data::<AudioBufferList>(property_address)
            .context("failed to get stream configuration")?;

        for i in 0..buffers.mNumberBuffers {
            let buf = buffers.mBuffers[i as usize];
            if buf.mNumberChannels > 0 {
                return Ok(buf.mNumberChannels);
            }
        }
        Ok(0)
    }

    pub fn get_output_channels(&self) -> Result<u32> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreamConfiguration,
            mScope: kAudioObjectPropertyScopeOutput,
            mElement: kAudioObjectPropertyElementWildcard,
        };

        self.get_channels(property_address)
            .context("failed to get stream configuration")
    }

    pub fn get_input_channels(&self) -> Result<u32> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreamConfiguration,
            mScope: kAudioObjectPropertyScopeInput,
            mElement: kAudioObjectPropertyElementWildcard,
        };

        self.get_channels(property_address)
            .context("failed to get stream configuration")
    }
}
