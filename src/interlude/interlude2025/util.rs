use super::constants::*;

pub fn weighting_curve(x: f64) -> f64 {
    0.002 + x.powi(4)
}

pub fn ms_to_jack_bpm(delta_ms: f64) -> f64 {
    (15000.0 / delta_ms).min(JACK_CURVE_CUTOFF)
}

pub fn ms_to_stream_bpm(delta_ms: f64) -> f64 {
    let result = 300.0 / (0.02 * delta_ms)
        - 300.0 / (0.02 * delta_ms).powf(STREAM_CURVE_CUTOFF) / STREAM_CURVE_CUTOFF2;
    result.max(0.0)
}

pub fn jack_compensation(jack_delta: f64, stream_delta: f64) -> f64 {
    if stream_delta <= 0.0 {
        return 1.0;
    }

    let ratio = jack_delta / stream_delta;
    let log_ratio = ratio.log2();
    log_ratio.max(0.0).sqrt().min(1.0)
}

pub fn calculate_note_total(j: f64, sl: f64, sr: f64) -> f64 {
    (STREAM_SCALE * sl.powf(STREAM_POW))
        .powf(OHT_NERF)
        .add((STREAM_SCALE * sr.powf(STREAM_POW)).powf(OHT_NERF))
        .add(j.powf(OHT_NERF))
        .powf(1.0 / OHT_NERF)
}

trait FloatExt {
    fn add(self, other: Self) -> Self;
}
impl FloatExt for f64 {
    fn add(self, other: Self) -> Self {
        self + other
    }
}

pub fn strain_func(half_life_ms: f64, current_value: f64, input: f64, delta_ms: f64) -> f64 {
    let decay_rate = 0.5f64.ln() / half_life_ms;
    let decay = (decay_rate * delta_ms.min(STRAIN_TIME_CAP)).exp();
    let time_cap_decay = if delta_ms > STRAIN_TIME_CAP {
        (decay_rate * (delta_ms - STRAIN_TIME_CAP)).exp()
    } else {
        1.0
    };

    let a = current_value * time_cap_decay;
    let b = input * input * STRAIN_SCALE;
    b - (b - a) * decay
}

pub fn weighted_overall_difficulty(data: &[f64]) -> f64 {
    let mut data_array: Vec<f64> = data.iter().copied().filter(|&x| x > 0.0).collect();
    data_array.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    if data_array.is_empty() {
        return 0.0;
    }

    let length = data_array.len() as f64;
    let mut weight = 0.0;
    let mut total = 0.0;

    for (i, &value) in data_array.iter().enumerate() {
        let position = (i as f64 + MOST_IMPORTANT_NOTES - length) / MOST_IMPORTANT_NOTES;
        let x = position.max(0.0);

        let w = weighting_curve(x);
        weight += w;
        total += value * w;
    }

    if weight <= 0.0 {
        return 0.0;
    }

    let weighted_average = total / weight;
    let result = weighted_average.powf(CURVE_POWER) * CURVE_SCALE;

    if result.is_finite() {
        result
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interlude::interlude2025::constants::*;
    use approx::assert_relative_eq;

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
