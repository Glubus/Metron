use super::util::*;
use rhythm_open_exchange::RoxChart;

#[derive(Debug)]
pub struct Interlude2025Difficulty {
    pub stars: f64,
}

impl crate::calculator::Rating for Interlude2025Difficulty {}

pub fn calculate(chart: &RoxChart, rate: f32) -> f64 {
    if chart.notes.is_empty() {
        return 0.0;
    }

    let key_count = chart.key_count() as usize;
    if key_count == 0 {
        return 0.0;
    }

    let mut notes_by_time: std::collections::BTreeMap<i64, Vec<(usize, bool)>> =
        std::collections::BTreeMap::new();

    for note in &chart.notes {
        let is_hold = note.duration_us() > 0;
        let col = note.column as usize;
        notes_by_time
            .entry(note.time_us)
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

        let mut note_difficulties = vec![0.0f64; key_count];
        let mut row_strains = vec![0.0f64; key_count];

        for k in 0..key_count {
            let has_note = notes.iter().any(|(col, _)| *col == k);

            if has_note {
                let jack_delta = time - last_note_in_column[k];
                let j = if jack_delta > 0.0 {
                    ms_to_jack_bpm(jack_delta)
                } else {
                    0.0
                };

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
                            let trill_v = ms_to_stream_bpm(trill_delta)
                                * jack_compensation(jack_delta, trill_delta);
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
