use rhythm_open_exchange::RoxChart;
use crate::calculator::{Calculator, CalculatorResult};

// Constants
const CURVE_POWER: f64 = 0.6;
const CURVE_SCALE: f64 = 0.4056;
const MOST_IMPORTANT_NOTES: f64 = 2500.0;

const JACK_CURVE_CUTOFF: f64 = 230.0;
const STREAM_CURVE_CUTOFF: f64 = 10.0;
const STREAM_CURVE_CUTOFF2: f64 = 10.0;
const OHT_NERF: f64 = 3.0;
const STREAM_SCALE: f64 = 6.0;
const STREAM_POW: f64 = 0.5;

const STRAIN_SCALE: f64 = 0.01626;
const STRAIN_TIME_CAP: f64 = 200.0;

fn weighting_curve(x: f64) -> f64 {
    0.002 + x.powi(4)
}

fn ms_to_jack_bpm(delta_ms: f64) -> f64 {
    (15000.0 / delta_ms).min(JACK_CURVE_CUTOFF)
}

fn ms_to_stream_bpm(delta_ms: f64) -> f64 {
    let result = 300.0 / (0.02 * delta_ms)
        - 300.0 / (0.02 * delta_ms).powf(STREAM_CURVE_CUTOFF) / STREAM_CURVE_CUTOFF2;
    result.max(0.0)
}

fn jack_compensation(jack_delta: f64, stream_delta: f64) -> f64 {
    if stream_delta <= 0.0 {
        return 1.0;
    }

    let ratio = jack_delta / stream_delta;
    let log_ratio = ratio.log2();
    log_ratio.max(0.0).sqrt().min(1.0)
}

fn calculate_note_total(j: f64, sl: f64, sr: f64) -> f64 {
    (STREAM_SCALE * sl.powf(STREAM_POW)).powf(OHT_NERF)
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

fn strain_func(half_life_ms: f64, current_value: f64, input: f64, delta_ms: f64) -> f64 {
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

fn weighted_overall_difficulty(data: &[f64]) -> f64 {
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

pub fn calculate_difficulty(chart: &RoxChart, rate: f32) -> f64 {
    if chart.notes.is_empty() {
        return 0.0;
    }

    let key_count = chart.key_count() as usize;
    if key_count == 0 {
         return 0.0;
    }

    let mut notes_by_time: std::collections::BTreeMap<i64, Vec<(usize, bool)>> = std::collections::BTreeMap::new();

    for note in &chart.notes {
        let is_hold = note.duration_us() > 0;
        let col = note.column as usize;
        notes_by_time.entry(note.time_us)
            .or_default()
            .push((col, is_hold));
    }
    
    if notes_by_time.is_empty() {
        return 0.0;
    }

    let mut last_note_in_column = vec![0.0f64; key_count];
    let mut strain_values = vec![0.0f64; key_count];
    let mut strain_data_points = Vec::new();

    let hand_split = key_count / 2;
    let rate = rate as f64;

    for (&time_us, notes) in &notes_by_time {
        let time = (time_us as f64 / 1000.0) / rate;
        
        // BPM lookup logic omitted as established it was unused in original C#

        let mut note_difficulties = vec![0.0f64; key_count];
        let mut row_strains = vec![0.0f64; key_count];

        for k in 0..key_count {
            let has_note = notes.iter().any(|(col, _)| *col == k);

            if has_note {
                let jack_delta = time - last_note_in_column[k];
                let j = if jack_delta > 0.0 { ms_to_jack_bpm(jack_delta) } else { 0.0 };

                let (hand_lo, hand_hi) = if k < hand_split {
                    (0, hand_split - 1)
                } else {
                    (hand_split, key_count - 1)
                };

                let mut sl: f64 = 0.0;
                let mut sr: f64 = 0.0;

                for hand_k in hand_lo..=hand_hi {
                    if hand_k != k {
                        let trill_delta = time - last_note_in_column[hand_k];
                         if trill_delta > 0.0 {
                            let trill_v = ms_to_stream_bpm(trill_delta) * jack_compensation(jack_delta, trill_delta);
                            if hand_k < k {
                                sl = sl.max(trill_v);
                            } else {
                                sr = sr.max(trill_v);
                            }
                        }
                    }
                }

                note_difficulties[k] = calculate_note_total(j, sl, sr);
                
                let input = note_difficulties[k];
                let delta = jack_delta.max(0.0);
                
                strain_values[k] = strain_func(1575.0, strain_values[k], input, delta);
                row_strains[k] = strain_values[k];
                
                last_note_in_column[k] = time;
            }
        }
        
        for &strain in &row_strains {
            if strain > 0.0 {
                strain_data_points.push(strain);
            }
        }
    }

    weighted_overall_difficulty(&strain_data_points)
}

#[derive(Debug, Default)]
pub struct Interlude2025DifficultyContext {
    pub clock_rate: Option<u32>,
}

#[derive(Debug, Default)]
pub struct Interlude2025PerformanceContext {
    pub accuracy: f32,
}

#[derive(Debug)]
pub struct Interlude2025Difficulty {
    pub stars: f64,
}

impl crate::calculator::Rating for Interlude2025Difficulty {}

#[derive(Debug)]
pub struct Interlude2025Performance {
    pub pp: f64,
}

impl crate::calculator::Rating for Interlude2025Performance {}

pub struct Interlude2025;

impl Calculator for Interlude2025 {
    type DifficultyContext = Interlude2025DifficultyContext;
    type PerformanceContext = Interlude2025PerformanceContext;

    type Difficulty = Interlude2025Difficulty;
    type Performance = Interlude2025Performance;

    const NAME: &'static str = "Interlude 2025";
    const VERSION: &'static str = "2025.1";
    const GAME: &'static str = "osu!mania";
    const YEAR: u32 = 2025;

    fn calculate_difficulty(
        &self,
        chart: &RoxChart,
        context: &Self::DifficultyContext,
    ) -> CalculatorResult<Self::Difficulty> {
        let rate = context.clock_rate.unwrap_or(100) as f32 / 100.0;
        let stars = calculate_difficulty(chart, rate);
        Ok(Interlude2025Difficulty { stars })
    }

    fn calculate_performance(
        &self,
        _chart: &RoxChart,
        _difficulty: &Self::Difficulty,
        _context: &Self::PerformanceContext,
    ) -> CalculatorResult<Self::Performance> {
        // Placeholder
        Ok(Interlude2025Performance { pp: 0.0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
