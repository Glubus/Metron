use crate::interlude::interlude2025::Interlude2025DifficultyContext;

use super::util::{
    calculate_note_total, jack_compensation, ms_to_jack_bpm, ms_to_stream_bpm, strain_func,
    weighted_overall_difficulty,
};
use rhythm_open_exchange::RoxChart;

#[derive(Debug)]
pub struct Interlude2025Difficulty {
    pub stars: f64,
}

impl crate::calculator::Rating for Interlude2025Difficulty {}

fn trill_contribution_for_hand(
    hand_k: usize,
    column: usize,
    time: f64,
    last_note_in_column: &[f64],
    jack_delta: f64,
) -> (f64, f64) {
    if hand_k == column {
        return (0.0, 0.0);
    }
    let trill_delta = time - last_note_in_column[hand_k];
    if trill_delta <= 0.0 {
        return (0.0, 0.0);
    }
    let trill_v = ms_to_stream_bpm(trill_delta) * jack_compensation(jack_delta, trill_delta);
    if hand_k < column { (trill_v, 0.0) } else { (0.0, trill_v) }
}

fn calculate_trill_difficulty(
    column: usize,
    time: f64,
    last_note_in_column: &[f64],
    hand_range: std::ops::RangeInclusive<usize>,
    jack_delta: f64,
) -> (f64, f64) {
    let mut sl = 0.0f64;
    let mut sr = 0.0f64;
    for hand_k in hand_range {
        let (s, r) = trill_contribution_for_hand(hand_k, column, time, last_note_in_column, jack_delta);
        sl = sl.max(s);
        sr = sr.max(r);
    }
    (sl, sr)
}

fn calculate_column_difficulty(
    column: usize,
    time: f64,
    last_note_time: f64,
    current_strain: f64,
    last_note_in_column: &[f64],
    hand_split: usize,
    key_count: usize,
) -> (f64, f64) {
    let jack_delta = time - last_note_time;
    let j = if jack_delta > 0.0 { ms_to_jack_bpm(jack_delta) } else { 0.0 };

    let hand_range = if column < hand_split {
        0..=hand_split - 1
    } else {
        hand_split..=key_count - 1
    };

    let (sl, sr) =
        calculate_trill_difficulty(column, time, last_note_in_column, hand_range, jack_delta);

    let note_difficulty = calculate_note_total(j, sl, sr);
    let updated_strain = strain_func(1575.0, current_strain, note_difficulty, jack_delta.max(0.0));

    (note_difficulty, updated_strain)
}

fn calculate_and_record_column_strain(
    col: usize,
    time: f64,
    last_note_in_column: &mut [f64],
    strain_values: &mut [f64],
    strain_data_points: &mut Vec<f64>,
    hand_split: usize,
    key_count: usize,
) {
    let (_, updated_strain) = calculate_column_difficulty(
        col, time, last_note_in_column[col], strain_values[col],
        last_note_in_column, hand_split, key_count,
    );
    strain_values[col] = updated_strain;
    last_note_in_column[col] = time;
    if updated_strain > 0.0 {
        strain_data_points.push(updated_strain);
    }
}

fn sort_chart_notes(chart: &RoxChart) -> Vec<(i64, usize)> {
    let mut notes: Vec<(i64, usize)> = chart
        .notes
        .iter()
        .map(|n| (n.time_us, n.column as usize))
        .collect();
    notes.sort_unstable_by_key(|&(t, col)| (t, col));
    notes
}

#[must_use]
pub fn calculate(chart: &RoxChart, context: &Interlude2025DifficultyContext) -> f64 {
    if chart.notes.is_empty() {
        return 0.0;
    }

    let key_count = chart.key_count() as usize;
    if key_count == 0 {
        return 0.0;
    }

    let sorted_notes = sort_chart_notes(chart);
    let mut last_note_in_column = vec![0.0f64; key_count];
    let mut strain_values = vec![0.0f64; key_count];
    let mut strain_data_points = Vec::with_capacity(chart.notes.len());

    let hand_split = key_count / 2;
    let rate = f64::from(context.clock_rate.unwrap_or_default());

    let mut i = 0;
    while i < sorted_notes.len() {
        let current_time_us = sorted_notes[i].0;
        #[allow(clippy::cast_precision_loss)]
        let time = (current_time_us as f64 / 1000.0) / rate;

        while i < sorted_notes.len() && sorted_notes[i].0 == current_time_us {
            let col = sorted_notes[i].1;
            calculate_and_record_column_strain(col, time, &mut last_note_in_column, &mut strain_values, &mut strain_data_points, hand_split, key_count);
            i += 1;
        }
    }

    weighted_overall_difficulty(strain_data_points)
}
