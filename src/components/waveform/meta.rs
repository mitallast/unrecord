use crate::time::TimeCode;
use anyhow::{Result, anyhow};
use gpui::SharedString;
use hound::WavSpec;
use std::path::Path;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct WaveClipParameter {
    key: SharedString,
    value: SharedString,
}

#[allow(dead_code)]
impl WaveClipParameter {
    fn new(key: impl Into<Arc<str>>, value: impl Into<Arc<str>>) -> Self {
        Self {
            key: SharedString::new(key),
            value: SharedString::new(value),
        }
    }

    pub fn key(&self) -> SharedString {
        self.key.clone()
    }

    pub fn value(&self) -> SharedString {
        self.value.clone()
    }
}

#[derive(Clone)]
pub struct WaveClipMetadata {
    filename: SharedString,
    filepath: SharedString,
    spec: WavSpec,
    sample_count: usize,
    frame_count: usize,
    bitrate_kbps: u64,
    duration_millis: u64,
    peak_dbfs: f64,
    rms_dbfs: f64,
    integrated_lufs: f64,
    crest_factor_db: f64,
    dc_offset_percent: f64,
}

#[allow(dead_code)]
impl WaveClipMetadata {
    pub fn new(filepath: &Path, spec: WavSpec, samples: &Vec<f32>) -> Result<Self> {
        let filename = filepath
            .file_name()
            .ok_or(anyhow!("no filename"))?
            .to_string_lossy()
            .into_owned()
            .into();

        let filepath = filepath.as_os_str().to_string_lossy().into_owned().into();

        let provider = WaveClipMetadataProvider { spec: &spec, samples };

        Ok(Self {
            filename,
            filepath,
            spec,
            sample_count: samples.len(),
            frame_count: provider.frame_count(),
            bitrate_kbps: provider.bitrate_kbps(),
            duration_millis: provider.duration_millis(),
            peak_dbfs: provider.peak_dbfs(),
            rms_dbfs: provider.rms_dbfs(),
            integrated_lufs: provider.integrated_lufs(),
            crest_factor_db: provider.crest_factor_db(),
            dc_offset_percent: provider.dc_offset_percent(),
        })
    }

    #[allow(dead_code)]
    pub fn info(&self) -> Vec<WaveClipParameter> {
        vec![
            WaveClipParameter::new("Filename", self.filename.clone()),
            WaveClipParameter::new("Channels", format!("{}", self.spec.channels)),
            WaveClipParameter::new("Sample rate", format!("{}", self.spec.sample_rate)),
            WaveClipParameter::new("Bit Depth", format!("{}-bit PCM", self.spec.bits_per_sample)),
            WaveClipParameter::new("Bitrate", format!("{} kbps", self.bitrate_kbps)),
            WaveClipParameter::new("Sample count", format!("{}", self.sample_count)),
            WaveClipParameter::new("Duration", TimeCode::from_millis(self.duration_millis).to_string()),
            WaveClipParameter::new("Peak", format!("{:.2} dBFS", self.peak_dbfs)),
            WaveClipParameter::new("RMS", format!("{:.2} dBFS", self.rms_dbfs)),
            WaveClipParameter::new("LUFS int", format!("{:.2} dBFS", self.integrated_lufs)),
            WaveClipParameter::new("Crest Factor", format!("{:.2} dB", self.crest_factor_db)),
            WaveClipParameter::new("DC offset", format!("{:.2}%", self.dc_offset_percent)),
        ]
    }

    pub fn filename(&self) -> SharedString {
        self.filename.clone()
    }

    pub fn filepath(&self) -> SharedString {
        self.filepath.clone()
    }

    pub fn spec(&self) -> WavSpec {
        self.spec
    }

    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    pub fn bitrate_kbps(&self) -> u64 {
        self.bitrate_kbps
    }

    pub fn duration_millis(&self) -> u64 {
        self.duration_millis
    }

    pub fn peak_dbfs(&self) -> f64 {
        self.peak_dbfs
    }

    pub fn rms_dbfs(&self) -> f64 {
        self.rms_dbfs
    }

    pub fn integrated_lufs(&self) -> f64 {
        self.integrated_lufs
    }

    pub fn crest_factor_db(&self) -> f64 {
        self.crest_factor_db
    }

    pub fn dc_offset_percent(&self) -> f64 {
        self.dc_offset_percent
    }
}

struct WaveClipMetadataProvider<'a> {
    spec: &'a WavSpec,
    samples: &'a Vec<f32>,
}

impl<'a> WaveClipMetadataProvider<'a> {
    fn frame_count(&self) -> usize {
        self.samples.len() / self.spec.channels as usize
    }

    fn duration_millis(&self) -> u64 {
        let millis = self.frame_count() as f64 * 1000.0 / self.spec.sample_rate as f64;
        millis as u64
    }

    fn peak_amplitude(&self) -> f32 {
        self.samples.iter().fold(0.0_f32, |max, sample| max.max(sample.abs()))
    }

    fn peak_dbfs(&self) -> f64 {
        Self::amplitude_to_dbfs(self.peak_amplitude() as f64)
    }

    fn rms_amplitude(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let power_sum = self
            .samples
            .iter()
            .fold(0.0_f64, |sum, sample| sum + (*sample as f64) * (*sample as f64));
        (power_sum / self.samples.len() as f64).sqrt()
    }

    fn rms_dbfs(&self) -> f64 {
        Self::amplitude_to_dbfs(self.rms_amplitude())
    }

    fn crest_factor_db(&self) -> f64 {
        self.peak_dbfs() - self.rms_dbfs()
    }

    /// Integrated loudness estimate in LUFS.
    /// Uses BS.1770-style 400 ms blocks with absolute/relative gating,
    /// but without K-weighting filter.
    fn integrated_lufs(&self) -> f64 {
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

        let rel_gated: Vec<f64> = abs_gated.into_iter().filter(|power| *power >= rel_gate_power).collect();
        if rel_gated.is_empty() {
            return f64::NEG_INFINITY;
        }

        let integrated_power = rel_gated.iter().sum::<f64>() / rel_gated.len() as f64;
        Self::lufs_from_power(integrated_power)
    }

    fn dc_offset_percent(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum = self.samples.iter().fold(0.0_f64, |acc, sample| acc + *sample as f64);
        (sum / self.samples.len() as f64).abs() * 100.0
    }

    fn bitrate_kbps(&self) -> u64 {
        ((self.spec.sample_rate as f64 * self.spec.channels as f64 * self.spec.bits_per_sample as f64) / 1000.0).round()
            as u64
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
