use crate::audio::device::CoreAudioDevice;
use anyhow::{Context, Result, anyhow};
use coreaudio::audio_unit::macos_helpers::{get_audio_device_ids, get_default_device_id};

pub struct CoreAudioDriver;

impl CoreAudioDriver {
    pub fn get_default_device(&self, input: bool) -> Result<CoreAudioDevice> {
        let device_id = get_default_device_id(input) //
            .ok_or(anyhow!("failed to get default device id"))?;
        Ok(CoreAudioDevice::from_id(device_id))
    }

    pub fn list_devices(&self) -> Result<Vec<CoreAudioDevice>> {
        let device_ids = get_audio_device_ids().context("failed to get audio device ids")?;

        let devices: Vec<CoreAudioDevice> = device_ids
            .into_iter()
            .map(|device_id| CoreAudioDevice::from_id(device_id))
            .collect();

        Ok(devices)
    }
}
