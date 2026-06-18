/// Internal note representation with times in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Note {
    pub column: usize,
    pub hit_time: i64,
}
