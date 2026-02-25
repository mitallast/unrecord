use log::info;
use std::hash::{DefaultHasher, Hasher};

pub fn make_impulse_test_f32_stereo_interleaved(
    pre_silence_frames: usize,
    post_silence_frames: usize,
    amp_l: f32,
    amp_r: f32,
) -> Vec<f32> {
    assert!(amp_l >= 0.0 && amp_r >= 0.0);
    let total_frames = pre_silence_frames + 1 + post_silence_frames;
    let mut x = vec![0f32; total_frames * 2];

    let idx = pre_silence_frames * 2;
    x[idx] = amp_l; // L
    x[idx + 1] = amp_r; // R
    x
}

fn frame_value_f32(interleaved: &[f32], frame: usize) -> f32 {
    let l = interleaved[2 * frame];
    let r = interleaved[2 * frame + 1];
    if l.abs() >= r.abs() { l } else { r }
}

/// Возвращает latency в frames (целое число)
pub fn estimate_latency_by_peak_in_window_f32_stereo_interleaved_frames(
    recorded: &[f32],
    pre_silence_frames: usize,
    max_search_frames: usize, // сколько кадров после старта смотреть
) -> Option<isize> {
    if recorded.len() < 2 {
        return None;
    }
    let frames_total = recorded.len() / 2;

    // окно поиска: [0 ... min(frames_total, pre_silence_frames + max_search_frames)]
    let end = (pre_silence_frames + max_search_frames).min(frames_total);
    if end <= 1 {
        return None;
    }

    let mut best_frame = 0usize;
    let mut best_amp = 0f32;

    for frame in 0..end {
        let amp = frame_value_f32(recorded, frame).abs();
        if amp > best_amp {
            best_amp = amp;
            best_frame = frame;
        }
    }

    Some(best_frame as isize - pre_silence_frames as isize)
}

pub fn sample_stats(samples: &Vec<f32>) {
    let min = samples.iter().fold(0.0, |a, s| f32::min(a, *s));
    let max = samples.iter().fold(0.0, |a, s| f32::max(a, *s));

    // compute simple hash sum
    let mut hasher = DefaultHasher::new();
    for x in samples {
        let sample = (*x as f64 * i32::MAX as f64) as i64;
        hasher.write_i64(sample);
    }
    let hash = hasher.finish();

    let sqr_sum = samples.iter().fold(0.0f64, |sqr_sum, s| {
        let sample = *s as f64;
        sqr_sum + sample * sample
    });
    let rms = sqr_sum / samples.len() as f64;
    let len = samples.len();

    let lead_zeroes = samples.iter().take_while(|&&x| x == 0.0).count();

    info!("min={min} max={max} rms={rms} hash={hash} samples={len} lead_zeroes={lead_zeroes}");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_estimate_latency_by_peak_in_window_f32_stereo_interleaved_frames() {
        let silence_frames = 64;
        let peak_amplitude = 0.7;

        let samples = make_impulse_test_f32_stereo_interleaved(
            silence_frames,
            silence_frames,
            peak_amplitude,
            peak_amplitude,
        );

        let latency = estimate_latency_by_peak_in_window_f32_stereo_interleaved_frames(
            &samples,
            silence_frames,
            silence_frames,
        );

        assert_eq!(latency, Some(0));
    }
}
