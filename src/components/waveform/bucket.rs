// read-only structure
pub struct WaveFormBucket {
    pub min: f32,
    pub max: f32,
}

const MIN: f32 = f32::INFINITY;
const MAX: f32 = f32::NEG_INFINITY;

impl WaveFormBucket {
    pub fn from_buckets(chunk: &[WaveFormBucket]) -> Self {
        let min = chunk.iter().fold(MIN, |m, b| f32::min(m, b.min));
        let max = chunk.iter().fold(MAX, |m, b| f32::max(m, b.max));
        Self { min, max }
    }

    pub fn from_samples(chunk: &[f32]) -> Self {
        let min = chunk.iter().fold(MIN, |m, b| f32::min(m, *b));
        let max = chunk.iter().fold(MAX, |m, b| f32::max(m, *b));
        Self { min, max }
    }
}
