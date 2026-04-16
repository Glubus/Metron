use super::note::Note;

/// Parsed map data with times in milliseconds.
#[derive(Debug, Clone)]
pub struct MapData {
    /// Hit leniency parameter (derived from OD)
    pub hit_leniency: f64,
    /// Number of columns
    pub column_count: usize,
    /// Total duration in milliseconds
    pub total_duration: i64,
    /// All notes sorted by hit time
    pub notes: Vec<Note>,
    /// Notes organized by column
    pub notes_by_column: Vec<Vec<Note>>,
    /// Long notes only
    pub long_notes: Vec<Note>,
    /// Tail sequence sorted by end time
    pub tail_sequence: Vec<Note>,
    /// Long notes organized by column
    pub long_notes_by_column: Vec<Vec<Note>>,
    /// Overall difficulty
    pub overall_difficulty: f64,
}

impl MapData {
    pub fn new() -> Self {
        Self {
            hit_leniency: 0.0,
            column_count: 0,
            total_duration: 0,
            notes: Vec::new(),
            notes_by_column: Vec::new(),
            long_notes: Vec::new(),
            tail_sequence: Vec::new(),
            long_notes_by_column: Vec::new(),
            overall_difficulty: 0.0,
        }
    }
}

impl Default for MapData {
    fn default() -> Self {
        Self::new()
    }
}
