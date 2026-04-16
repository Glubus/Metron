use super::grid::PatternGrid;
use super::merger::PatternMerger;
use super::types::PatternType;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, Deserialize)]
pub struct PatternTimelineEntry {
    pub start_time: i64,
    pub end_time: i64,
    pub duration: i64,
    pub pattern_type: PatternType,
    pub avg_bpm: f64,
    pub min_bpm: f64,
    pub max_bpm: f64,
    pub note_count: usize,
    pub segment_indices: Vec<usize>,
}

impl Serialize for PatternTimelineEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PatternTimelineEntry", 8)?;
        state.serialize_field("start_time", &self.start_time)?;
        state.serialize_field("end_time", &self.end_time)?;
        state.serialize_field("duration", &self.duration)?;
        state.serialize_field("pattern_type", &self.pattern_type.as_str())?;
        state.serialize_field("avg_bpm", &self.avg_bpm)?;
        state.serialize_field("min_bpm", &self.min_bpm)?;
        state.serialize_field("max_bpm", &self.max_bpm)?;
        state.serialize_field("note_count", &self.note_count)?;
        state.end()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternTimeline {
    pub entries: Vec<PatternTimelineEntry>,
}

impl PatternTimeline {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn build_from_cross_analysis(
        cross_analysis: &[(i64, i64, PatternType, f64)],
        grids: &[PatternGrid],
        timestamps: &[Vec<i64>],
        key_count: usize,
    ) -> Self {
        if cross_analysis.is_empty() {
            return Self::new();
        }

        let mut merged_entries: Vec<PatternTimelineEntry> = Vec::new();

        let (mut current_start, mut current_end, mut current_pattern, first_bpm) = cross_analysis[0];
        let mut current_bpms = if first_bpm > 0.0 { vec![first_bpm] } else { vec![] };

        for &(start, end, pattern, bpm) in cross_analysis.iter().skip(1) {
            if PatternMerger::are_compatible(current_pattern, pattern) {
                current_end = end;
                if bpm > 0.0 { current_bpms.push(bpm); }
                current_pattern = PatternMerger::get_dominant_pattern(current_pattern, pattern);
            } else {
                merged_entries.push(Self::create_entry(
                    current_start, current_end, current_pattern,
                    &current_bpms, grids, timestamps, key_count,
                ));
                current_start = start;
                current_end = end;
                current_pattern = pattern;
                current_bpms = if bpm > 0.0 { vec![bpm] } else { vec![] };
            }
        }

        merged_entries.push(Self::create_entry(
            current_start, current_end, current_pattern,
            &current_bpms, grids, timestamps, key_count,
        ));

        Self { entries: merged_entries }
    }

    fn create_entry(
        start: i64,
        end: i64,
        pattern: PatternType,
        bpms: &[f64],
        grids: &[PatternGrid],
        timestamps: &[Vec<i64>],
        key_count: usize,
    ) -> PatternTimelineEntry {
        let avg_bpm = if bpms.is_empty() { 0.0 } else { bpms.iter().sum::<f64>() / bpms.len() as f64 };
        let min_bpm = bpms.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_bpm = bpms.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let mut note_count = 0;
        let mut segment_indices = Vec::new();

        for (seg_idx, times) in timestamps.iter().enumerate() {
            if !times.is_empty() && times[0] <= end && times[times.len() - 1] >= start {
                segment_indices.push(seg_idx);
                let grid = &grids[seg_idx];
                for (t_idx, &t) in times.iter().enumerate() {
                    if t >= start && t <= end {
                        for col in 0..key_count {
                            if grid.get_note(t_idx, col) { note_count += 1; }
                        }
                    }
                }
            }
        }

        PatternTimelineEntry {
            start_time: start,
            end_time: end,
            duration: end - start,
            pattern_type: pattern,
            avg_bpm,
            min_bpm: if min_bpm == f64::INFINITY { 0.0 } else { min_bpm },
            max_bpm: if max_bpm == f64::NEG_INFINITY { 0.0 } else { max_bpm },
            note_count,
            segment_indices,
        }
    }
}

impl Default for PatternTimeline {
    fn default() -> Self {
        Self::new()
    }
}
