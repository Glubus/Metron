use super::constants::{
    CURVE_POWER, CURVE_SCALE, JACK_CURVE_CUTOFF, MOST_IMPORTANT_NOTES, STRAIN_SCALE,
    STRAIN_TIME_CAP, STREAM_CURVE_CUTOFF2, STREAM_SCALE,
};

// ln(0.5) = -LN_2, precomputed to avoid recomputing per note in strain_func.
const LN_HALF: f64 = -std::f64::consts::LN_2;

// Decay rate for strain_func — always called with half_life_ms = 1575.0.
const STRAIN_DECAY_RATE: f64 = LN_HALF / 1575.0;

#[must_use]
pub fn weighting_curve(x: f64) -> f64 {
    0.002 + x.powi(4)
}

#[must_use]
pub fn ms_to_jack_bpm(delta_ms: f64) -> f64 {
    (15000.0 / delta_ms).min(JACK_CURVE_CUTOFF)
}

#[must_use]
pub fn ms_to_stream_bpm(delta_ms: f64) -> f64 {
    let x = 0.02 * delta_ms;
    // STREAM_CURVE_CUTOFF = 10.0 → powi(10) instead of powf(10.0)
    let result = 300.0 / x - 300.0 / x.powi(10) / STREAM_CURVE_CUTOFF2;
    result.max(0.0)
}

#[must_use]
pub fn jack_compensation(jack_delta: f64, stream_delta: f64) -> f64 {
    if stream_delta <= 0.0 {
        return 1.0;
    }
    let log_ratio = (jack_delta / stream_delta).log2();
    log_ratio.max(0.0).sqrt().min(1.0)
}

#[must_use]
pub fn calculate_note_total(j: f64, sl: f64, sr: f64) -> f64 {
    // STREAM_POW = 0.5  → sqrt()
    // OHT_NERF   = 3.0  → powi(3) and cbrt()
    // STREAM_SCALE = 6.0
    let sl_term = (STREAM_SCALE * sl.sqrt()).powi(3);
    let sr_term = (STREAM_SCALE * sr.sqrt()).powi(3);
    let j_term  = j.powi(3);
    (sl_term + sr_term + j_term).cbrt()
}

/// Exponential strain decay. `half_life_ms` is always 1575.0 — decay rate precomputed.
#[must_use]
pub fn strain_func(current_value: f64, input: f64, delta_ms: f64) -> f64 {
    let decay = (STRAIN_DECAY_RATE * delta_ms.min(STRAIN_TIME_CAP)).exp();
    let time_cap_decay = if delta_ms > STRAIN_TIME_CAP {
        (STRAIN_DECAY_RATE * (delta_ms - STRAIN_TIME_CAP)).exp()
    } else {
        1.0
    };
    let a = current_value * time_cap_decay;
    let b = input * input * STRAIN_SCALE;
    b - (b - a) * decay
}

fn accumulate_weighted(i: usize, value: f64, length: f64, weight: &mut f64, total: &mut f64) {
    #[allow(clippy::cast_precision_loss)]
    let position = (i as f64 + MOST_IMPORTANT_NOTES - length) / MOST_IMPORTANT_NOTES;
    let w = weighting_curve(position.max(0.0));
    *weight += w;
    *total += value * w;
}

/// Takes ownership of `data` to sort in-place, avoiding an extra allocation.
/// Data is expected to already be filtered (no zeros) by the caller.
#[must_use]
pub fn weighted_overall_difficulty(mut data: Vec<f64>) -> f64 {
    data.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    if data.is_empty() {
        return 0.0;
    }

    #[allow(clippy::cast_precision_loss)]
    let length = data.len() as f64;
    let mut weight = 0.0;
    let mut total = 0.0;

    for (i, &value) in data.iter().enumerate() {
        accumulate_weighted(i, value, length, &mut weight, &mut total);
    }

    if weight <= 0.0 {
        return 0.0;
    }

    let result = (total / weight).powf(CURVE_POWER) * CURVE_SCALE;
    if result.is_finite() { result } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use super::super::constants::JACK_CURVE_CUTOFF;

    #[test]
    fn test_weighting_curve() {
        assert_relative_eq!(weighting_curve(0.0), 0.002);
        assert_relative_eq!(weighting_curve(1.0), 1.002);
        assert_relative_eq!(weighting_curve(0.5), 0.002 + 0.5f64.powi(4));
    }

    #[test]
    fn test_ms_to_jack_bpm() {
        assert_relative_eq!(ms_to_jack_bpm(100.0), 150.0);
        assert_relative_eq!(ms_to_jack_bpm(10.0), JACK_CURVE_CUTOFF);
    }

    #[test]
    fn test_ms_to_stream_bpm() {
        let res = ms_to_stream_bpm(100.0);
        assert!(res > 149.0 && res < 150.0);
    }

    #[test]
    fn test_jack_compensation() {
        assert_relative_eq!(jack_compensation(100.0, 100.0), 0.0);
        assert_relative_eq!(jack_compensation(400.0, 100.0), 1.0);
        assert_relative_eq!(jack_compensation(200.0, 100.0), 1.0);
    }
}
