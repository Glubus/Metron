use crate::calculator::CalculatorError;
use minacalc_rs::Note;
use rhythm_open_exchange::{NoteType, RoxChart};

fn add_column_to_current_row(notes: &mut Vec<Note>, column_bit: u32) {
    if let Some(row) = notes.last_mut() {
        row.notes |= column_bit;
    }
}

fn start_new_row(notes: &mut Vec<Note>, time_us: i64, column_bit: u32) {
    notes.push(Note { notes: column_bit, row_time: time_us as f32 / 1_000_000.0 });
}

fn merge_sorted_pairs_into_notes(pairs: Vec<(i64, u32)>) -> Vec<Note> {
    let mut notes: Vec<Note> = Vec::with_capacity(pairs.len());
    let mut last_time_us: i64 = i64::MIN;
    for (time_us, column_bit) in pairs {
        if time_us == last_time_us {
            add_column_to_current_row(&mut notes, column_bit);
        } else {
            start_new_row(&mut notes, time_us, column_bit);
            last_time_us = time_us;
        }
    }
    notes
}

/// Converts a ROX chart to MinaCalc notes at the chart's native timestamps.
///
/// Notes at the same timestamp are merged via bitmask OR, matching Etterna's
/// internal row representation.
///
/// MinaCalc receives the desired music rate separately in `calc_at_rate` and
/// applies that rate internally. Pre-scaling timestamps here would apply the
/// requested rate twice.
///
/// Uses sort + linear merge instead of HashMap to avoid hashing overhead.
pub fn chart_to_notes(chart: &RoxChart) -> Result<Vec<Note>, CalculatorError> {
    if chart.notes.is_empty() {
        return Err(CalculatorError::Calculation("Chart has no notes".into()));
    }

    // Collect playable (time, column_bit) pairs, then sort by time. Mines are
    // not playable notes and must not contribute to MinaCalc rows.
    let mut pairs: Vec<(i64, u32)> = chart
        .notes
        .iter()
        .filter(|n| !matches!(n.note_type, NoteType::Mine))
        .map(|n| (n.time_us, 1u32 << n.column))
        .collect();
    if pairs.is_empty() {
        return Err(CalculatorError::Calculation("Chart has no playable notes".into()));
    }
    pairs.sort_unstable_by_key(|&(t, _)| t);

    Ok(merge_sorted_pairs_into_notes(pairs))
}
