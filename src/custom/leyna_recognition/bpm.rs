use rox::RoxChart;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct TimingInfo {
    pub timestamp: i64,
    pub delta_to_next: i64,
    pub bpm: f64,
    pub note_count: usize,
}

pub struct TimingAnalyzer {
    pub timing_info: Vec<TimingInfo>,
}

impl TimingAnalyzer {
    pub fn new(chart: &RoxChart, ignore_holds: bool) -> Self {
        let notes = Self::get_sorted_notes(chart, ignore_holds);
        let time_groups = Self::group_notes_by_time(&notes);
        let sorted_times: Vec<i64> = time_groups.keys().cloned().collect();
        let info_list = Self::calculate_timing_info(&sorted_times, &time_groups);
        Self { timing_info: info_list }
    }

    fn get_sorted_notes(chart: &RoxChart, ignore_holds: bool) -> Vec<&rox::Note> {
        let mut notes: Vec<_> = chart.notes.iter()
            .filter(|n| !ignore_holds || n.duration_us() == 0)
            .collect();
        notes.sort_by_key(|n| n.time_us);
        notes
    }

    fn group_notes_by_time(notes: &[&rox::Note]) -> BTreeMap<i64, usize> {
        let mut time_groups = BTreeMap::new();
        for note in notes {
            *time_groups.entry(note.time_us).or_default() += 1;
        }
        time_groups
    }

    fn calculate_timing_info(times: &[i64], groups: &BTreeMap<i64, usize>) -> Vec<TimingInfo> {
        let mut list = Vec::with_capacity(times.len());
        for (i, &time) in times.iter().enumerate() {
            let delta = if i < times.len() - 1 { times[i + 1] - time } else { 0 };
            let bpm = if delta > 0 { 15_000_000.0 / delta as f64 } else { 0.0 };
            list.push(TimingInfo {
                timestamp: time,
                delta_to_next: delta,
                bpm,
                note_count: groups[&time],
            });
        }
        list
    }
}
