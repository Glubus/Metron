use rhythm_open_exchange::{NoteType, RoxChart};

use super::note::Note;

pub const MAX_SUPPORTED_KEYS: usize = 10;

#[derive(Debug)]
pub struct MapData {
    pub hit_leniency: f64,
    pub column_count: usize,
    pub total_duration: i64,
    pub notes: Vec<Note>,
    pub notes_by_column: Vec<Vec<Note>>,
}

pub fn preprocess_chart(chart: &RoxChart, clock_rate: f64, overall_difficulty: f64) -> MapData {
    let mut notes = Vec::with_capacity(chart.notes.len());
    for note in &chart.notes {
        if matches!(note.note_type, NoteType::Mine) {
            continue;
        }

        let hit_time_ms = (note.time_us as f64 / 1_000.0 / clock_rate) as i64;
        notes.push(Note {
            column: usize::from(note.column),
            hit_time: hit_time_ms,
        });
    }

    notes.sort_unstable();

    let column_count = usize::from(chart.key_count);
    let mut notes_by_column = vec![Vec::new(); column_count];
    for note in &notes {
        notes_by_column[note.column].push(*note);
    }

    let total_duration = notes.last().map_or(0, |note| note.hit_time + 1);
    MapData {
        hit_leniency: compute_hit_leniency(overall_difficulty),
        column_count,
        total_duration,
        notes,
        notes_by_column,
    }
}

pub fn compute_hit_leniency(overall_difficulty: f64) -> f64 {
    let x = 0.3 * ((64.5 - (overall_difficulty * 3.0).ceil()) / 500.0).sqrt();
    x.min(0.6 * (x - 0.09) + 0.09)
}
