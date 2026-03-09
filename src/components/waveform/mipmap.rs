use crate::components::waveform::bucket::WaveFormBucket;

// read-only structure
pub struct WaveFormMipMap {
    frames_per_bucket: usize,
    buckets: Vec<WaveFormBucket>,
}

impl WaveFormMipMap {
    pub fn from_samples(frames_per_bucket: usize, samples: &Vec<f32>) -> Self {
        let buckets: Vec<WaveFormBucket> = samples
            .chunks(frames_per_bucket)
            .map(WaveFormBucket::from_samples)
            .collect();

        Self {
            frames_per_bucket,
            buckets,
        }
    }

    pub fn from_level(level: &WaveFormMipMap, reduce_buckets: usize) -> Self {
        let frames_per_bucket = level.frames_per_bucket * reduce_buckets;

        let buckets: Vec<WaveFormBucket> = level
            .buckets
            .chunks(reduce_buckets)
            .map(WaveFormBucket::from_buckets)
            .collect();

        Self {
            frames_per_bucket,
            buckets,
        }
    }

    pub fn frames_per_bucket(&self) -> usize {
        self.frames_per_bucket
    }

    pub fn buckets_len(&self) -> usize {
        self.buckets.len()
    }

    pub fn min_max_for_frames(&self, start_frame: usize, end_frame: usize) -> Option<(f32, f32)> {
        if start_frame >= end_frame || self.buckets.is_empty() {
            return None;
        }

        let bucket_start = start_frame / self.frames_per_bucket;
        let bucket_end = end_frame.div_ceil(self.frames_per_bucket).min(self.buckets.len());

        if bucket_start >= bucket_end {
            return None;
        }

        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;

        for bucket in &self.buckets[bucket_start..bucket_end] {
            min = min.min(bucket.min);
            max = max.max(bucket.max);
        }

        Some((min, max))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_reduce_samples_10() {
        let samples = vec![0., 1., 2., 3., 4., 5., 6., 7., 8., 9.];
        let level = WaveFormMipMap::from_samples(10, &samples);

        assert_eq!(level.frames_per_bucket, 10);
        assert_eq!(level.buckets.len(), 1);
        assert_eq!(level.buckets[0].min, 0f32);
        assert_eq!(level.buckets[0].max, 9f32);
    }

    #[test]
    fn test_reduce_samples_5() {
        let samples = vec![0., 1., 2., 3., 4., 5., 6., 7., 8., 9.];
        let level = WaveFormMipMap::from_samples(5, &samples);

        assert_eq!(level.frames_per_bucket, 5);
        assert_eq!(level.buckets.len(), 2);
        assert_eq!(level.buckets[0].min, 0f32);
        assert_eq!(level.buckets[0].max, 4f32);
        assert_eq!(level.buckets[1].min, 5f32);
        assert_eq!(level.buckets[1].max, 9f32);
    }

    #[test]
    fn test_reduce_samples_3() {
        let samples = vec![0., 1., 2., 3., 4., 5., 6., 7., 8., 9.];
        let level = WaveFormMipMap::from_samples(3, &samples);

        assert_eq!(level.frames_per_bucket, 3);
        assert_eq!(level.buckets.len(), 4);
        assert_eq!(level.buckets[0].min, 0f32);
        assert_eq!(level.buckets[0].max, 2f32);
        assert_eq!(level.buckets[1].min, 3f32);
        assert_eq!(level.buckets[1].max, 5f32);
        assert_eq!(level.buckets[2].min, 6f32);
        assert_eq!(level.buckets[2].max, 8f32);
        assert_eq!(level.buckets[3].min, 9f32);
        assert_eq!(level.buckets[3].max, 9f32);
    }

    #[test]
    fn test_reduce_samples_2() {
        let samples = vec![0., 1., 2., 3., 4., 5., 6., 7., 8., 9.];
        let level = WaveFormMipMap::from_samples(2, &samples);

        assert_eq!(level.frames_per_bucket, 2);
        assert_eq!(level.buckets.len(), 5);
        assert_eq!(level.buckets[0].min, 0f32);
        assert_eq!(level.buckets[0].max, 1f32);
        assert_eq!(level.buckets[1].min, 2f32);
        assert_eq!(level.buckets[1].max, 3f32);
        assert_eq!(level.buckets[2].min, 4f32);
        assert_eq!(level.buckets[2].max, 5f32);
        assert_eq!(level.buckets[3].min, 6f32);
        assert_eq!(level.buckets[3].max, 7f32);
        assert_eq!(level.buckets[4].min, 8f32);
        assert_eq!(level.buckets[4].max, 9f32);
    }

    #[test]
    fn test_min_max_for_frames() {
        let samples = vec![0., 1., 2., 3., 4., 5., 6., 7., 8., 9.];
        let level = WaveFormMipMap::from_samples(2, &samples);

        let (min, max) = level.min_max_for_frames(3, 9).unwrap();
        assert_eq!(min, 2.0);
        assert_eq!(max, 9.0);
    }
}
