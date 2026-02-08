use crate::interlude::interlude2025::Interlude2025DifficultyContext;

use super::util::{
    calculate_note_total, jack_compensation, ms_to_jack_bpm, ms_to_stream_bpm, strain_func,
    weighted_overall_difficulty,
};
use rhythm_open_exchange::RoxChart;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Interlude2025Difficulty {
    pub stars: f64,
}

impl crate::calculator::Rating for Interlude2025Difficulty {}
/// Groups notes by timestamp for processing.
///
/// # Arguments
///
/// * `chart` - The chart containing notes to group
///
/// # Returns
///
/// `BTreeMap` where key is `time_us` and value is `Vec` of (`column`, `is_hold`) tuples
fn group_notes_by_time(chart: &RoxChart) -> BTreeMap<i64, Vec<(usize, bool)>> {
    let mut notes_by_time: BTreeMap<i64, Vec<(usize, bool)>> = BTreeMap::new();
    for note in &chart.notes {
        let is_hold = note.duration_us() > 0;
        let col = note.column as usize;
        notes_by_time
            .entry(note.time_us)
            .or_default()
            .push((col, is_hold));
    }
    notes_by_time
}

/// Calculates left and right trill/stream difficulty for a column.
///
/// # Arguments
///
/// * `column` - Current column index
/// * `time` - Current time in ms
/// * `last_note_in_column` - Last note time for each column
/// * `hand_range` - Range of columns in the same hand
/// * `jack_delta` - Time delta for jack detection
///
/// # Returns
///
/// (`left_stream_difficulty`, `right_stream_difficulty`)
fn calculate_trill_difficulty(
    column: usize,
    time: f64,
    last_note_in_column: &[f64],
    hand_range: std::ops::RangeInclusive<usize>,
    jack_delta: f64,
) -> (f64, f64) {
    let mut sl: f64 = 0.0;
    let mut sr: f64 = 0.0;

    for hand_k in hand_range {
        if hand_k != column {
            let trill_delta = time - last_note_in_column[hand_k];
            if trill_delta > 0.0 {
                let trill_v =
                    ms_to_stream_bpm(trill_delta) * jack_compensation(jack_delta, trill_delta);
                if hand_k < column {
                    sl = sl.max(trill_v);
                } else {
                    sr = sr.max(trill_v);
                }
            }
        }
    }

    (sl, sr)
}

/// Calculates difficulty and updated strain for a single column.
///
/// # Arguments
///
/// * `column` - Column index
/// * `time` - Current time in ms
/// * `last_note_time` - Last note time in this column
/// * `current_strain` - Current strain value for this column
/// * `last_note_in_column` - Last note times for all columns
/// * `hand_split` - Index that splits left/right hands
/// * `key_count` - Total number of columns
///
/// # Returns
///
/// (`note_difficulty`, `updated_strain_value`)
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
    let j = if jack_delta > 0.0 {
        ms_to_jack_bpm(jack_delta)
    } else {
        0.0
    };

    let hand_range = if column < hand_split {
        0..=hand_split - 1
    } else {
        hand_split..=key_count - 1
    };

    let (sl, sr) =
        calculate_trill_difficulty(column, time, last_note_in_column, hand_range, jack_delta);

    let note_difficulty = calculate_note_total(j, sl, sr);
    let delta = jack_delta.max(0.0);
    let updated_strain = strain_func(1575.0, current_strain, note_difficulty, delta);

    (note_difficulty, updated_strain)
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

    let notes_by_time = group_notes_by_time(chart);
    if notes_by_time.is_empty() {
        return 0.0;
    }

    let mut last_note_in_column = vec![0.0f64; key_count];
    let mut strain_values = vec![0.0f64; key_count];
    let mut strain_data_points = Vec::new();

    let hand_split = key_count / 2;
    let rate = f64::from(context.clock_rate.unwrap_or_default());

    for (&time_us, notes) in &notes_by_time {
        #[allow(clippy::cast_precision_loss)]
        let time = (time_us as f64 / 1000.0) / rate;
        let mut row_strains = vec![0.0f64; key_count];

        for k in 0..key_count {
            let has_note = notes.iter().any(|(col, _)| *col == k);
            if has_note {
                let (_, updated_strain) = calculate_column_difficulty(
                    k,
                    time,
                    last_note_in_column[k],
                    strain_values[k],
                    &last_note_in_column,
                    hand_split,
                    key_count,
                );

                strain_values[k] = updated_strain;
                row_strains[k] = updated_strain;
                last_note_in_column[k] = time;
            }
        }

        strain_data_points.extend(row_strains.iter().filter(|&&s| s > 0.0));
    }

    weighted_overall_difficulty(&strain_data_points)
}
