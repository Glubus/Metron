use super::grid::PatternGrid;
use super::types::PatternType;
use super::bpm::TimingAnalyzer;

pub struct CrossSegmentAnalyzer<'a> {
    grids: &'a [PatternGrid],
    timestamps: &'a [Vec<i64>],
    timing: &'a TimingAnalyzer,
    key_count: usize,
    unified_times: Vec<i64>,
    unified_notes: Vec<Vec<bool>>,
}

impl<'a> CrossSegmentAnalyzer<'a> {
    pub fn new(
        grids: &'a [PatternGrid],
        timestamps: &'a [Vec<i64>],
        timing: &'a TimingAnalyzer,
        key_count: usize,
    ) -> Self {
        let mut analyzer = Self {
            grids,
            timestamps,
            timing,
            key_count,
            unified_times: Vec::new(),
            unified_notes: Vec::new(),
        };
        analyzer.build_unified_timeline();
        analyzer
    }

    fn build_unified_timeline(&mut self) {
        for (grid, times) in self.grids.iter().zip(self.timestamps) {
            for (t_idx, &time) in times.iter().enumerate() {
                self.unified_times.push(time);
                let mut row = vec![false; self.key_count];
                for col in 0..self.key_count {
                    if grid.get_note(t_idx, col) { row[col] = true; }
                }
                self.unified_notes.push(row);
            }
        }
    }

    pub fn get_rolling_window(&self, center_idx: usize, window_size: usize) -> &[Vec<bool>] {
        let half = window_size / 2;
        let start = center_idx.saturating_sub(half);
        let end = (center_idx + half + window_size % 2).min(self.unified_notes.len());
        if start >= end { &[] } else { &self.unified_notes[start..end] }
    }

    pub fn classify_window(&self, window: &[Vec<bool>]) -> PatternType {
        if window.len() < 2 { return PatternType::VerySparse; }
        let total_notes = Self::count_notes(window);
        if total_notes == 0 { return PatternType::EmptyRegion; }
        if self.is_dense_chord(window, total_notes) { return PatternType::DenseChord; }
        let has_jumps = self.detect_jumps(window);
        let has_jacks = self.detect_jacks(window);
        let has_stream = self.detect_stream(window);
        self.determine_pattern_type(total_notes, window.len(), has_jumps, has_jacks, has_stream)
    }

    fn count_notes(window: &[Vec<bool>]) -> usize {
        window.iter().map(|row| row.iter().filter(|&&b| b).count()).sum()
    }

    fn is_dense_chord(&self, window: &[Vec<bool>], total_notes: usize) -> bool {
        let threshold = (window.len() * self.key_count) as f64 * 0.75;
        total_notes as f64 >= threshold
    }

    fn detect_jumps(&self, window: &[Vec<bool>]) -> bool {
        window.iter().any(|row| row.iter().filter(|&&b| b).count() >= 2)
    }

    fn detect_jacks(&self, window: &[Vec<bool>]) -> bool {
        for col in 0..self.key_count {
            let mut consecutive = 0;
            for row in window {
                if col < row.len() && row[col] {
                    consecutive += 1;
                    if consecutive >= 2 { return true; }
                } else {
                    consecutive = 0;
                }
            }
        }
        false
    }

    fn detect_stream(&self, window: &[Vec<bool>]) -> bool {
        for t in 0..window.len().saturating_sub(1) {
            let cols_t = Self::get_active_cols(&window[t]);
            let cols_next = Self::get_active_cols(&window[t + 1]);
            if !cols_t.is_empty() && !cols_next.is_empty() {
                if cols_t.iter().any(|c1| cols_next.iter().any(|c2| (*c1 as i32 - *c2 as i32).abs() == 1)) {
                    return true;
                }
            }
        }
        false
    }

    fn get_active_cols(row: &[bool]) -> Vec<usize> {
        row.iter().enumerate().filter(|(_, b)| **b).map(|(i, _)| i).collect()
    }

    fn determine_pattern_type(
        &self,
        total_notes: usize,
        window_len: usize,
        has_jumps: bool,
        has_jacks: bool,
        has_stream: bool,
    ) -> PatternType {
        if has_jumps && has_jacks { return PatternType::Chordjack; }
        if has_jumps && has_stream { return PatternType::Jumpstream; }
        if has_jacks && has_stream { return PatternType::Handstream; }
        if has_jumps { return PatternType::JumpSection; }
        if has_jacks { return PatternType::JackSection; }
        if has_stream { return PatternType::Stream; }
        if total_notes <= window_len { return PatternType::Light; }
        PatternType::Mixed
    }

    pub fn analyze_cross_segment(&self, window_size: usize) -> Vec<(i64, i64, PatternType, f64)> {
        let mut results = Vec::new();
        let mut i = 0;

        while i < self.unified_times.len() {
            let window = self.get_rolling_window(i, window_size);
            let pattern = self.classify_window(window);

            let end_idx = (i + window_size).min(self.unified_times.len());
            let start_time = self.unified_times[i];
            let end_time = if end_idx > 0 { self.unified_times[end_idx - 1] } else { start_time };

            let start_time_idx = self.timing.timing_info.partition_point(|tp| tp.timestamp < start_time);
            let mut window_bpms = Vec::new();
            for idx in start_time_idx..self.timing.timing_info.len() {
                let tp = &self.timing.timing_info[idx];
                if tp.timestamp > end_time { break; }
                if tp.bpm > 0.0 { window_bpms.push(tp.bpm); }
            }

            let avg_bpm = if window_bpms.is_empty() { 0.0 } else {
                window_bpms.iter().sum::<f64>() / window_bpms.len() as f64
            };

            results.push((start_time, end_time, pattern, avg_bpm));
            i += window_size / 2;
        }

        results
    }
}
