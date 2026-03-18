use crate::calculator::CalculatorError;
use minacalc_rs::Note;
use rhythm_open_exchange::RoxChart;
use std::collections::HashMap;

/// Converts a ROX chart to MinaCalc notes, applying an optional clock rate.
///
/// Notes at the same timestamp are merged via bitmask OR, matching Etterna's
/// internal row representation.
pub fn chart_to_notes(chart: &RoxChart, clock_rate: f64) -> Result<Vec<Note>, CalculatorError> {
    if chart.notes.is_empty() {
        return Err(CalculatorError::Calculation("Chart has no notes".into()));
    }

    let mut time_notes: HashMap<i64, u32> = HashMap::new();

    for note in &chart.notes {
        let scaled_time_us = (note.time_us as f64 / clock_rate) as i64;
        let column_bit = 1u32 << note.column;
        time_notes
            .entry(scaled_time_us)
            .and_modify(|bits| *bits |= column_bit)
            .or_insert(column_bit);
    }

    let mut notes: Vec<Note> = time_notes
        .into_iter()
        .map(|(time_us, notes)| Note {
            notes,
            row_time: time_us as f32 / 1_000_000.0,
        })
        .collect();

    notes.sort_by(|a, b| a.row_time.partial_cmp(&b.row_time).unwrap());

    Ok(notes)
}
