pub mod bars;
pub mod calculations;
pub mod interpolation;
pub mod map_data;
pub mod note;
pub mod process;
pub mod smoothing;
pub mod sums;
pub mod utils;

use crate::calculator::{CalculatorResult, Rating};
use rhythm_open_exchange::RoxChart;

pub use map_data::MapData;
pub use note::Note as InternalNote;

use super::SunnyxxyDifficultyContext;

#[derive(Debug)]
pub struct SunnyxxyDifficulty {
    pub stars: f64,
}

impl Rating for SunnyxxyDifficulty {}

pub fn calculate(chart: &RoxChart, context: &SunnyxxyDifficultyContext) -> CalculatorResult<SunnyxxyDifficulty> {
    let clock_rate = f64::from(context.clock_rate.unwrap_or_default());
    let od = f64::from(context.overall_difficulty.unwrap_or(8.0));
    let map_data = build_map_data(chart, clock_rate, od);
    let stars = process::calculate_internal(&map_data);
    Ok(SunnyxxyDifficulty { stars })
}

fn compute_hit_leniency(overall_difficulty: f64) -> f64 {
    let x = 0.3 * ((64.5 - (overall_difficulty * 3.0).ceil()) / 500.0).sqrt();
    x.min(0.6 * (x - 0.09) + 0.09)
}

fn chart_to_notes(chart: &RoxChart, clock_rate: f64) -> Vec<note::Note> {
    let mut notes: Vec<note::Note> = chart.notes.iter().map(|n| {
        let hit_ms = (n.time_us as f64 / 1_000.0 / clock_rate) as i64;
        let duration_us = n.duration_us();
        let tail_ms = if duration_us > 0 {
            ((n.time_us as f64 + duration_us as f64) / 1_000.0 / clock_rate) as i64
        } else {
            -1
        };
        note::Note::new(n.column as usize, hit_ms, tail_ms)
    }).collect();
    notes.sort_unstable_by_key(|n| n.hit_time);
    notes
}

fn group_by_column(notes: &[note::Note], k: usize) -> Vec<Vec<note::Note>> {
    let mut by_col = vec![Vec::new(); k];
    for n in notes { by_col[n.column].push(*n); }
    by_col
}

fn build_map_data(chart: &RoxChart, clock_rate: f64, overall_difficulty: f64) -> MapData {
    let k = chart.key_count() as usize;
    let notes = chart_to_notes(chart, clock_rate);
    let long_notes: Vec<note::Note> = notes.iter().filter(|n| n.is_long_note()).copied().collect();
    let notes_by_column = group_by_column(&notes, k);
    let long_notes_by_column = group_by_column(&long_notes, k);
    let times_by_column: Vec<Vec<i64>> = notes_by_column.iter()
        .map(|col| col.iter().map(|n| n.hit_time).collect())
        .collect();
    let mut tail_sequence = long_notes.clone();
    tail_sequence.sort_unstable_by_key(|n| n.tail_time);
    let total_duration = notes.iter()
        .map(|n| n.hit_time.max(n.tail_time))
        .max().unwrap_or(0) + 1;
    MapData {
        hit_leniency: compute_hit_leniency(overall_difficulty),
        column_count: k,
        total_duration,
        notes,
        notes_by_column,
        long_notes,
        tail_sequence,
        long_notes_by_column,
        times_by_column,
        overall_difficulty,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hit_leniency_od8_no_reduction() {
        // od=8: x = 0.3*sqrt(40.5/500) ≈ 0.08538; complement = 0.08723 > x → no reduction
        let x = compute_hit_leniency(8.0);
        let expected = 0.3 * ((64.5 - 24.0) / 500.0_f64).sqrt();
        assert!((x - expected).abs() < 1e-6, "expected {expected}, got {x}");
    }

    #[test]
    fn test_compute_hit_leniency_od0_gets_reduced() {
        // od=0: x = 0.3*sqrt(64.5/500) ≈ 0.10775; complement ≈ 0.10065 < x → reduced
        let x = compute_hit_leniency(0.0);
        let raw = 0.3 * (64.5_f64 / 500.0).sqrt();
        let complement = 0.6 * (raw - 0.09) + 0.09;
        assert!(x < raw, "expected reduction from {raw}");
        assert!((x - complement).abs() < 1e-9);
    }

    #[test]
    fn test_compute_hit_leniency_od10() {
        // od=10: ceil(30) = 30, x = 0.3*sqrt(34.5/500)
        let x = compute_hit_leniency(10.0);
        let expected = 0.3 * (34.5_f64 / 500.0).sqrt();
        assert!((x - expected).abs() < 1e-6);
    }

    #[test]
    fn test_compute_hit_leniency_positive() {
        for od in [0.0, 4.0, 8.0, 10.0] {
            let x = compute_hit_leniency(od);
            assert!(x > 0.0, "hit_leniency should be positive for od={od}");
        }
    }

    #[test]
    fn test_group_by_column_distributes_correctly() {
        let notes = vec![
            note::Note::simple(0, 100),
            note::Note::simple(1, 200),
            note::Note::simple(0, 300),
        ];
        let by_col = group_by_column(&notes, 2);
        assert_eq!(by_col[0].len(), 2);
        assert_eq!(by_col[1].len(), 1);
        assert_eq!(by_col[0][0].hit_time, 100);
        assert_eq!(by_col[0][1].hit_time, 300);
    }

    #[test]
    fn test_group_by_column_empty() {
        let by_col = group_by_column(&[], 4);
        assert_eq!(by_col.len(), 4);
        assert!(by_col.iter().all(|c| c.is_empty()));
    }
}
