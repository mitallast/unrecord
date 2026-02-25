use crate::audio::read_file;

use crate::ui::TrackInfoItem;
use anyhow::Result;
use gpui::SharedString;
use hound::{SampleFormat, WavSpec};
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct SessionTrack {
    path: SharedString,
    filename: SharedString,
    spec: WavSpec,
    samples: Arc<Vec<f32>>,
}

impl SessionTrack {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let (spec, samples) = read_file(&path)?;

        let filename = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("No filename"))?
            .to_string_lossy()
            .to_string();

        let path = path.to_string_lossy().to_string();

        Ok(Self {
            path: SharedString::from(path),
            filename: SharedString::from(filename),
            spec,
            samples: Arc::new(samples),
        })
    }

    pub fn info(&self) -> Vec<TrackInfoItem> {
        vec![
            TrackInfoItem::new("Filename", self.filename.to_string()),
            TrackInfoItem::new("Channels", format!("{}", self.channels())),
            TrackInfoItem::new("Sample rate", format!("{}", self.sample_rate())),
            TrackInfoItem::new("Bit Depth", format!("{}-bit PCM", self.bits_per_sample())),
            TrackInfoItem::new("Bitrate", format!("{} kbps", self.bitrate_kbps())),
            TrackInfoItem::new("Sample count", format!("{}", self.sample_count())),
            TrackInfoItem::new("Duration", self.duration_label()),
            TrackInfoItem::new("Peak", format!("{:.2} dBFS", self.peak_dbfs())),
            TrackInfoItem::new("RMS", format!("{:.2} dBFS", self.rms_dbfs())),
            TrackInfoItem::new("LUFS int", format!("{:.2} dBFS", self.integrated_lufs())),
            TrackInfoItem::new("Crest Factor", format!("{:.2} dB", self.crest_factor_db())),
            TrackInfoItem::new("DC offset", format!("{:.2}%", self.dc_offset_percent())),
        ]
    }

    pub fn path(&self) -> SharedString {
        self.path.clone()
    }

    pub fn filename(&self) -> SharedString {
        self.filename.clone()
    }

    pub fn channels(&self) -> usize {
        self.spec.channels as usize
    }

    pub fn sample_rate(&self) -> usize {
        self.spec.sample_rate as usize
    }

    pub fn bits_per_sample(&self) -> usize {
        self.spec.bits_per_sample as usize
    }

    pub fn sample_format(&self) -> SampleFormat {
        self.spec.sample_format
    }

    pub fn samples(&self) -> Arc<Vec<f32>> {
        self.samples.clone()
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    pub fn frame_count(&self) -> usize {
        self.samples.len() / self.channels()
    }

    pub fn duration_seconds(&self) -> f64 {
        self.frame_count() as f64 / self.spec.sample_rate as f64
    }

    pub fn duration_label(&self) -> String {
        let total_millis = (self.duration_seconds() * 1000.0).round() as u64;
        let minutes = total_millis / 60_000;
        let seconds = (total_millis % 60_000) / 1000;
        let millis = total_millis % 1000;
        format!("{minutes:02}:{seconds:02}.{millis:03}")
    }

    pub fn peak_amplitude(&self) -> f32 {
        self.samples
            .iter()
            .fold(0.0_f32, |max, sample| max.max(sample.abs()))
    }

    pub fn peak_dbfs(&self) -> f64 {
        Self::amplitude_to_dbfs(self.peak_amplitude() as f64)
    }

    pub fn rms_amplitude(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let power_sum = self.samples.iter().fold(0.0_f64, |sum, sample| {
            sum + (*sample as f64) * (*sample as f64)
        });
        (power_sum / self.samples.len() as f64).sqrt()
    }

    pub fn rms_dbfs(&self) -> f64 {
        Self::amplitude_to_dbfs(self.rms_amplitude())
    }

    pub fn crest_factor_db(&self) -> f64 {
        self.peak_dbfs() - self.rms_dbfs()
    }

    /// Integrated loudness estimate in LUFS.
    /// Uses BS.1770-style 400 ms blocks with absolute/relative gating,
    /// but without K-weighting filter.
    pub fn integrated_lufs(&self) -> f64 {
        let frames = self.frame_count();
        if frames == 0 || self.spec.sample_rate == 0 {
            return f64::NEG_INFINITY;
        }

        let block_frames = ((self.spec.sample_rate as f64 * 0.4).round() as usize).max(1);
        let hop_frames = ((self.spec.sample_rate as f64 * 0.1).round() as usize).max(1);

        let mut block_powers = Vec::new();
        let mut start = 0usize;
        while start + block_frames <= frames {
            let mut power_sum = 0.0_f64;
            for frame in start..(start + block_frames) {
                let i = frame * self.spec.channels as usize;
                let l = self.samples[i] as f64;
                let r = self.samples[i + 1] as f64;
                power_sum += l * l + r * r;
            }
            block_powers.push(power_sum / block_frames as f64);
            start += hop_frames;
        }

        if block_powers.is_empty() {
            return f64::NEG_INFINITY;
        }

        let abs_gate_power = Self::power_from_lufs(-70.0);
        let abs_gated: Vec<f64> = block_powers
            .iter()
            .copied()
            .filter(|power| *power >= abs_gate_power)
            .collect();
        if abs_gated.is_empty() {
            return f64::NEG_INFINITY;
        }

        let mean_power = abs_gated.iter().sum::<f64>() / abs_gated.len() as f64;
        let ungated_lufs = Self::lufs_from_power(mean_power);
        let rel_gate_power = Self::power_from_lufs(ungated_lufs - 10.0);

        let rel_gated: Vec<f64> = abs_gated
            .into_iter()
            .filter(|power| *power >= rel_gate_power)
            .collect();
        if rel_gated.is_empty() {
            return f64::NEG_INFINITY;
        }

        let integrated_power = rel_gated.iter().sum::<f64>() / rel_gated.len() as f64;
        Self::lufs_from_power(integrated_power)
    }

    pub fn dc_offset_percent(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum = self
            .samples
            .iter()
            .fold(0.0_f64, |acc, sample| acc + *sample as f64);
        (sum / self.samples.len() as f64).abs() * 100.0
    }

    pub fn bitrate_kbps(&self) -> u64 {
        ((self.spec.sample_rate as f64
            * self.spec.channels as f64
            * self.spec.bits_per_sample as f64)
            / 1000.0)
            .round() as u64
    }

    fn amplitude_to_dbfs(amplitude: f64) -> f64 {
        if amplitude <= 0.0 {
            return f64::NEG_INFINITY;
        }
        20.0 * amplitude.log10()
    }

    fn lufs_from_power(power: f64) -> f64 {
        if power <= 0.0 {
            return f64::NEG_INFINITY;
        }
        -0.691 + 10.0 * power.log10()
    }

    fn power_from_lufs(lufs: f64) -> f64 {
        10.0_f64.powf((lufs + 0.691) / 10.0)
    }
}
