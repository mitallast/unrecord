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

fn rms_f32_as_f32_mono_from_interleaved(x: &[f32]) -> f32 {
    if x.len() < 2 {
        return 0.0;
    }
    let frames = x.len() / 2;

    let mut acc = 0.0f64;
    let mut n = 0usize;

    for i in 0..frames {
        let l = x[2 * i];
        let r = x[2 * i + 1];

        let v_f32 = if l.abs() >= r.abs() { l } else { r };

        let v = v_f32 as f64;
        acc += v * v;
        n += 1;
    }

    if n == 0 {
        0.0
    } else {
        ((acc / n as f64) as f32).sqrt()
    }
}

/// Оценка latency (в ФРЕЙМАХ) по interleaved stereo f32.
///
/// pre_silence_frames — сколько тишины (в кадрах) было до импульса в тестовом сигнале
/// noise_window_frames — сколько первых кадров использовать для оценки шумового пола (обычно = pre_silence_frames)
/// k_sigma — порог = k_sigma * RMS(noise)
/// min_run_frames — сколько подряд кадров должны превышать порог
pub fn estimate_latency_from_impulse_ir_f32_stereo_interleaved(
    recorded_interleaved: &[f32],
    pre_silence_frames: usize,
    noise_window_frames: usize,
    k_sigma: f32,
    min_run_frames: usize,
) -> Option<isize> {
    if recorded_interleaved.len() < 2 || min_run_frames == 0 {
        return None;
    }
    let frames_total = recorded_interleaved.len() / 2;

    let nw = noise_window_frames
        .min(frames_total)
        .min(pre_silence_frames.max(1));

    // RMS по шумовому окну (по кадрам)
    let noise_rms = rms_f32_as_f32_mono_from_interleaved(&recorded_interleaved[..nw * 2]);

    // thr в f32-единицах (по выбранному mode)
    let thr = (k_sigma * noise_rms).max(1.0);
    let thr_f32 = thr.ceil();

    let mut run = 0usize;
    let mut first_idx: Option<usize> = None;

    for frame in 0..frames_total {
        let l = recorded_interleaved[2 * frame];
        let r = recorded_interleaved[2 * frame + 1];

        let v_f32 = if l.abs() >= r.abs() { l } else { r };

        if v_f32.abs() >= thr_f32 {
            if run == 0 {
                first_idx = Some(frame);
            }
            run += 1;
            if run >= min_run_frames {
                let idx = first_idx.unwrap();
                // latency в кадрах: (где началась реакция) - (где был импульс)
                return Some(idx as isize - pre_silence_frames as isize);
            }
        } else {
            run = 0;
            first_idx = None;
        }
    }

    None
}

fn frame_value_f32(interleaved: &[f32], frame: usize) -> f32 {
    let l = interleaved[2 * frame];
    let r = interleaved[2 * frame + 1];
    if l.abs() >= r.abs() { l } else { r }
}

/// Находит пик |v| и возвращает (frame_index, frac_offset) где frac_offset в [-0.5..0.5] примерно.
fn peak_with_parabolic_refine(interleaved: &[f32]) -> Option<(usize, f32)> {
    if interleaved.len() < 6 {
        return None;
    } // минимум 3 frames
    let frames = interleaved.len() / 2;

    // 1) integer peak по |v|
    let mut best_i = 0usize;
    let mut best_a = 0f32;

    for i in 0..frames {
        let a = frame_value_f32(interleaved, i).abs();
        if a > best_a {
            best_a = a;
            best_i = i;
        }
    }

    // 2) параболическая интерполяция вокруг пика по |v|
    // delta = 0.5*(y_-1 - y_+1) / (y_-1 - 2*y0 + y_+1)
    // где y = |v|
    if best_i == 0 || best_i + 1 >= frames {
        return Some((best_i, 0.0));
    }

    let ym1 = frame_value_f32(interleaved, best_i - 1).abs();
    let y0 = frame_value_f32(interleaved, best_i).abs();
    let yp1 = frame_value_f32(interleaved, best_i + 1).abs();

    let denom = ym1 - 2.0 * y0 + yp1;
    if denom.abs() < 1e-12 {
        return Some((best_i, 0.0));
    }

    let delta = 0.5 * (ym1 - yp1) / denom;
    // ограничим на всякий случай
    let delta = delta.clamp(-0.5, 0.5);

    Some((best_i, delta))
}

/// latency относительно позиции импульса в отправленном сигнале (pre_silence_frames).
/// Возвращает latency в frames как f32 (с дробной частью).
pub fn estimate_latency_by_peak_f32_stereo_interleaved(
    recorded: &[f32],
    pre_silence_frames: usize,
) -> Option<f32> {
    let (peak_i, frac) = peak_with_parabolic_refine(recorded)?;
    Some((peak_i as f32 + frac) - pre_silence_frames as f32)
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
    fn test_estimate_latency_from_impulse_ir_f32_stereo_interleaved() {
        let silence_frames = 64;
        let peak_amplitude = 0.7;

        let samples = make_impulse_test_f32_stereo_interleaved(
            silence_frames,
            silence_frames,
            peak_amplitude,
            peak_amplitude,
        );

        let latency = estimate_latency_from_impulse_ir_f32_stereo_interleaved(
            &samples,
            silence_frames,
            silence_frames,
            20.0,
            1,
        );

        assert_eq!(latency, Some(0));
    }

    #[test]
    fn test_estimate_latency_by_peak_f32_stereo_interleaved() {
        let silence_frames = 64;
        let peak_amplitude = 0.7;

        let samples = make_impulse_test_f32_stereo_interleaved(
            silence_frames,
            silence_frames,
            peak_amplitude,
            peak_amplitude,
        );

        let latency = estimate_latency_by_peak_f32_stereo_interleaved(&samples, silence_frames);
        assert_eq!(latency, Some(0f32));
    }

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
