use crate::components::waveform::mipmap::WaveFormMipMap;
use std::sync::Arc;

const SAMPLES_PER_BUCKET: usize = 2;
const REDUCE_BUCKETS: usize = 2;
const LEVEL_MIN_LEN: usize = 32;

struct WaveFormInner {
    samples: Vec<f32>,
    mip_map: Vec<WaveFormMipMap>,
}

#[derive(Clone)]
pub struct WaveForm {
    inner: Arc<WaveFormInner>,
}

impl WaveFormInner {
    fn mip_level_for_frames_per_px(&self, frames_per_px: f32) -> Option<&WaveFormMipMap> {
        let target_samples = frames_per_px.max(1.0);
        self.mip_map
            .iter()
            .rev()
            .find(|level| level.frames_per_bucket() as f32 <= target_samples)
    }

    fn raw_min_max_for_frames(&self, start_frame: usize, end_frame: usize) -> Option<(f32, f32)> {
        if start_frame >= end_frame || start_frame >= self.samples.len() {
            return None;
        }

        let end_frame = end_frame.min(self.samples.len());
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for sample in &self.samples[start_frame..end_frame] {
            min = min.min(*sample);
            max = max.max(*sample);
        }
        Some((min, max))
    }

    fn min_max_for_frames(&self, start_frame: usize, end_frame: usize, frames_per_px: f32) -> Option<(f32, f32)> {
        let first_level = self.mip_map.first()?;
        let target_samples = frames_per_px.max(1.0);
        if target_samples < first_level.frames_per_bucket() as f32 {
            return self.raw_min_max_for_frames(start_frame, end_frame);
        }

        self.mip_level_for_frames_per_px(target_samples)
            .and_then(|level| level.min_max_for_frames(start_frame, end_frame))
            .or_else(|| self.raw_min_max_for_frames(start_frame, end_frame))
    }
}

impl WaveForm {
    pub fn from(samples: Vec<f32>) -> Self {
        Self::build(samples, SAMPLES_PER_BUCKET, REDUCE_BUCKETS, LEVEL_MIN_LEN)
    }

    pub fn build(samples: Vec<f32>, samples_per_bucket: usize, reduce_buckets: usize, level_min_len: usize) -> Self {
        let mut level = WaveFormMipMap::from_samples(samples_per_bucket, &samples);
        let mut mip_map = Vec::new();
        mip_map.push(level);
        while mip_map.last().unwrap().buckets_len() > level_min_len {
            level = WaveFormMipMap::from_level(&mip_map.last().unwrap(), reduce_buckets);
            mip_map.push(level);
        }
        let inner = WaveFormInner { samples, mip_map };
        Self { inner: Arc::new(inner) }
    }

    pub fn frames(&self) -> usize {
        self.inner.samples.len()
    }

    pub fn samples(&self) -> &[f32] {
        self.inner.samples.as_slice()
    }

    pub fn first_frames_per_bucket(&self) -> usize {
        self.inner.mip_map[0].frames_per_bucket()
    }

    pub fn min_max_for_frames(&self, start_frame: usize, end_frame: usize, frames_per_px: f32) -> Option<(f32, f32)> {
        self.inner.min_max_for_frames(start_frame, end_frame, frames_per_px)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_samples() {
        let samples = vec![1f32; 8 * 8 * 8];
        let channel = WaveForm::build(samples.clone(), 8, 8, 8);

        assert_eq!(channel.inner.samples, samples);
        assert_eq!(channel.inner.mip_map.len(), 2);
        assert_eq!(channel.inner.mip_map[0].frames_per_bucket(), 8);
        assert_eq!(channel.inner.mip_map[0].buckets_len(), 8 * 8);
        assert_eq!(channel.inner.mip_map[1].frames_per_bucket(), 8 * 8);
        assert_eq!(channel.inner.mip_map[1].buckets_len(), 8);
    }

    #[test]
    fn test_pick_mip_level() {
        let channel = WaveForm::build(vec![1f32; 8 * 8 * 8], 8, 8, 8);

        assert!(channel.inner.mip_level_for_frames_per_px(1.0).is_none());
        assert_eq!(
            channel
                .inner
                .mip_level_for_frames_per_px(8.0)
                .map(|l| l.frames_per_bucket()),
            Some(8)
        );
        assert_eq!(
            channel
                .inner
                .mip_level_for_frames_per_px(64.0)
                .map(|l| l.frames_per_bucket()),
            Some(64)
        );
        assert_eq!(
            channel
                .inner
                .mip_level_for_frames_per_px(1000.0)
                .map(|l| l.frames_per_bucket()),
            Some(64)
        );
    }

    #[test]
    fn test_min_max_fallback_to_raw_for_small_frames_per_px() {
        let channel = WaveForm::from(vec![0.0, 1.0, 2.0, 3.0, 10.0, 5.0, 6.0, 7.0, 8.0]);
        let (min, max) = channel.min_max_for_frames(4, 5, 1.0).unwrap();
        assert_eq!(min, 10.0);
        assert_eq!(max, 10.0);
    }
}
