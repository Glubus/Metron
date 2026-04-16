use std::fmt;

/// Represents a note in an osu! map (times in milliseconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Note {
    /// Column index (0-based)
    pub column: usize,
    /// Hit time in milliseconds
    pub hit_time: i64,
    /// End time in milliseconds (-1 for single notes)
    pub tail_time: i64,
}

impl Note {
    pub fn new(column: usize, hit_time: i64, tail_time: i64) -> Self {
        Self { column, hit_time, tail_time }
    }

    pub fn simple(column: usize, hit_time: i64) -> Self {
        Self { column, hit_time, tail_time: -1 }
    }

    pub fn long_note(column: usize, hit_time: i64, tail_time: i64) -> Self {
        Self { column, hit_time, tail_time }
    }

    pub fn is_long_note(&self) -> bool {
        self.tail_time >= 0
    }

    pub fn duration(&self) -> i64 {
        if self.is_long_note() {
            self.tail_time - self.hit_time
        } else {
            0
        }
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_long_note() {
            write!(f, "Note(col={}, hit={}ms, tail={}ms)", self.column, self.hit_time, self.tail_time)
        } else {
            write!(f, "Note(col={}, hit={}ms)", self.column, self.hit_time)
        }
    }
}
