use crate::audio::read_file;
use crate::components::waveform::form::WaveForm;
use crate::components::waveform::meta::WaveClipMetadata;
use anyhow::Result;
use std::path::Path;

#[derive(Clone)]
pub struct WaveClip {
    metadata: WaveClipMetadata,
    channels: Vec<WaveForm>,
}

impl WaveClip {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let (spec, samples) = read_file(path)?;
        let channel_count = spec.channels as usize;

        let metadata = WaveClipMetadata::new(path, spec, &samples)?;

        let mut channel_samples = vec![Vec::with_capacity(metadata.frame_count()); channel_count];

        for (index, sample) in samples.into_iter().enumerate() {
            channel_samples[index % channel_count].push(sample);
        }

        let channels: Vec<WaveForm> = channel_samples.into_iter().map(WaveForm::from).collect();

        Ok(Self { metadata, channels })
    }

    pub fn channels(&self) -> &[WaveForm] {
        &self.channels
    }

    pub fn frame_count(&self) -> usize {
        self.metadata.frame_count()
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn metadata(&self) -> &WaveClipMetadata {
        &self.metadata
    }
}
