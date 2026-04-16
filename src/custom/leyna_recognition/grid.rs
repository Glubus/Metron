use rhythm_open_exchange::RoxChart;

#[derive(Debug, Clone)]
pub struct PatternGrid {
    pub columns: usize,
    pub time_slices: usize,
    pub time_start_us: i64,
    pub time_end_us: i64,
    grid: Vec<bool>,
}

impl PatternGrid {
    pub fn new(columns: usize, time_slices: usize) -> Self {
        Self {
            columns,
            time_slices,
            time_start_us: 0,
            time_end_us: 0,
            grid: vec![false; columns * time_slices],
        }
    }

    pub fn set_note(&mut self, time_index: usize, column: usize, value: bool) {
        if time_index < self.time_slices && column < self.columns {
            self.grid[time_index * self.columns + column] = value;
        }
    }

    pub fn get_note(&self, time_index: usize, column: usize) -> bool {
        if time_index < self.time_slices && column < self.columns {
            self.grid[time_index * self.columns + column]
        } else {
            false
        }
    }

    pub fn note_count(&self) -> usize {
        self.grid.iter().filter(|&&b| b).count()
    }

    pub fn from_chart(
        chart: &RoxChart,
        max_time_slices: usize,
        ignore_holds: bool,
    ) -> (Vec<Self>, Vec<Vec<i64>>) {
        let mut notes: Vec<_> = chart.notes.iter()
            .filter(|n| !ignore_holds || n.duration_us() == 0)
            .collect();
        notes.sort_by_key(|n| n.time_us);

        if notes.is_empty() {
            return (vec![], vec![]);
        }

        let mut time_groups: std::collections::BTreeMap<i64, Vec<u8>> =
            std::collections::BTreeMap::new();
        for note in notes {
            time_groups.entry(note.time_us).or_default().push(note.column);
        }

        let sorted_times: Vec<i64> = time_groups.keys().cloned().collect();
        let total_time_slots = sorted_times.len();

        let mut grids = Vec::new();
        let mut all_timestamps = Vec::new();
        let mut segment_start = 0;

        let key_count = chart.key_count() as usize;

        while segment_start < total_time_slots {
            let segment_end = (segment_start + max_time_slices).min(total_time_slots);
            let segment_times = &sorted_times[segment_start..segment_end];

            let mut grid = Self::new(key_count, segment_times.len());
            grid.time_start_us = segment_times[0];
            grid.time_end_us = segment_times[segment_times.len() - 1];

            for (i, &time_val) in segment_times.iter().enumerate() {
                if let Some(columns) = time_groups.get(&time_val) {
                    for &col in columns {
                        grid.set_note(i, col as usize, true);
                    }
                }
            }

            grids.push(grid);
            all_timestamps.push(segment_times.to_vec());
            segment_start = segment_end;
        }

        (grids, all_timestamps)
    }
}
