//! Etterna's row grid: how note times reach MinaCalc.
//!
//! Etterna stores each note as an integer row, `lround(beat * 48)`, and places
//! every BPM breakpoint on a row too. A chart whose notes already sit on that
//! grid keeps its times. Any other chart is snapped onto Etterna's rows the way
//! its osu loader does it.

use rox::{NoteType, RoxChart, TimingPoint};

/// Etterna stores a note's position as `lround(beat * 48)`.
pub const ROWS_PER_BEAT: f64 = 48.0;

/// ROX rounds each time to whole microseconds, so two notes on the same grid
/// phase differ by at most this much.
pub const PHASE_TOLERANCE_US: f64 = 2.0;

/// A chart is already on rows when at most this share of its unique note times
/// is off the grid. Corpus: on-rows charts are at most 2.2% off; snapped osu
/// charts are at least 91.9% off.
pub const SNAP_THRESHOLD_PCT: f64 = 10.0;

/// How note times are handed to MinaCalc.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Rows {
    /// A chart already on Etterna's rows keeps its times. Any other chart is
    /// snapped onto them.
    #[default]
    Quantized,
    /// Times exactly as the chart gives them. No detection and no snapping.
    Exact,
}

/// A stretch of constant BPM, starting at `start_us` in chart time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub start_us: i64,
    pub bpm: f64,
}

impl Segment {
    /// Length of one row in µs.
    #[must_use]
    pub fn row_us(&self) -> f64 {
        60_000_000.0 / (self.bpm * ROWS_PER_BEAT)
    }
}

/// BPM segments as Etterna's osu loader builds them. An uninherited point sets
/// the BPM. An inherited (scroll-speed) point scales the last BPM by
/// `|beat_length / 100|`, which is `1 / |scroll_speed|` once ROX has converted
/// it. A point at the same time as the previous one replaces it.
#[must_use]
pub fn segments(chart: &RoxChart) -> Vec<Segment> {
    let mut timing: Vec<&TimingPoint> = chart.timing_points.iter().collect();
    timing.sort_by_key(|tp| tp.time_us());
    let mut out: Vec<Segment> = Vec::new();
    let mut base_bpm: Option<f64> = None;
    for tp in timing {
        let bpm = match tp {
            TimingPoint::Bpm { bpm, .. } => {
                base_bpm = Some(f64::from(*bpm));
                f64::from(*bpm)
            }
            TimingPoint::Sv { scroll_speed, .. } => {
                let Some(base) = base_bpm else { continue };
                let multiplier = f64::from(*scroll_speed).abs();
                if !multiplier.is_finite() || multiplier == 0.0 {
                    continue;
                }
                base / multiplier
            }
        };
        if !(bpm.is_finite() && bpm > 0.0) {
            continue;
        }
        let seg = Segment {
            start_us: tp.time_us().max(0),
            bpm,
        };
        match out.last_mut() {
            Some(last) if last.start_us == seg.start_us => *last = seg,
            _ => out.push(seg),
        }
    }
    out
}

/// Circular distance between two phases in `[0, period)`.
fn phase_distance(a: f64, b: f64, period: f64) -> f64 {
    let d = (a - b).abs();
    d.min(period - d)
}

/// Sorted, de-duplicated times of the playable notes. Mines are not playable;
/// chords share one time and count once.
#[must_use]
pub fn unique_times(chart: &RoxChart) -> Vec<i64> {
    let mut times: Vec<i64> = chart
        .notes
        .iter()
        .filter(|n| !matches!(n.note_type, NoteType::Mine))
        .map(|n| n.time_us)
        .collect();
    times.sort_unstable();
    times.dedup();
    times
}

/// Per-time placement: the segment containing it and its phase on that grid.
#[derive(Debug, Clone, Copy)]
struct Placement {
    segment: usize,
    phase_us: f64,
}

fn place(times: &[i64], segs: &[Segment]) -> Vec<Placement> {
    times
        .iter()
        .map(|&t| {
            let segment = segs.partition_point(|s| s.start_us <= t).saturating_sub(1);
            let seg = segs[segment];
            let phase_us = ((t - seg.start_us) as f64).rem_euclid(seg.row_us());
            Placement { segment, phase_us }
        })
        .collect()
}

/// For each unique time: whether it shares a grid phase with an adjacent time
/// in the same segment. Phases are measured from the segment start. A
/// constant-phase run matches; scattered notes do not.
#[must_use]
pub fn aligned_flags(times: &[i64], segs: &[Segment]) -> Vec<bool> {
    let placed = place(times, segs);
    let same_phase = |a: Placement, b: Placement| -> bool {
        a.segment == b.segment
            && phase_distance(a.phase_us, b.phase_us, segs[a.segment].row_us())
                <= PHASE_TOLERANCE_US
    };
    (0..placed.len())
        .map(|i| {
            let here = placed[i];
            let prev = i.checked_sub(1).map(|j| placed[j]);
            let next = placed.get(i + 1).copied();
            prev.is_some_and(|p| same_phase(here, p)) || next.is_some_and(|n| same_phase(here, n))
        })
        .collect()
}

/// Per-segment tally of the residual test.
#[derive(Debug, Clone, Copy)]
pub struct SegmentStat {
    pub index: usize,
    pub bpm: f64,
    pub row_us: f64,
    pub times: usize,
    pub unaligned: usize,
}

/// Chart-level residual tally.
#[derive(Debug, Clone)]
pub struct Diagnosis {
    pub keys: u8,
    pub notes: usize,
    pub times: usize,
    pub unaligned: usize,
    pub segments: Vec<SegmentStat>,
}

impl Diagnosis {
    #[must_use]
    pub fn unaligned_pct(&self) -> f64 {
        if self.times == 0 {
            0.0
        } else {
            100.0 * self.unaligned as f64 / self.times as f64
        }
    }
}

/// Runs the residual test over the whole chart, one grid per BPM segment.
#[must_use]
pub fn diagnose(chart: &RoxChart) -> Diagnosis {
    let segs = segments(chart);
    let times = unique_times(chart);
    let flags = if segs.is_empty() {
        vec![false; times.len()]
    } else {
        aligned_flags(&times, &segs)
    };
    let placed = if segs.is_empty() {
        Vec::new()
    } else {
        place(&times, &segs)
    };
    let mut stats: Vec<SegmentStat> = segs
        .iter()
        .enumerate()
        .map(|(index, s)| SegmentStat {
            index,
            bpm: s.bpm,
            row_us: s.row_us(),
            times: 0,
            unaligned: 0,
        })
        .collect();
    for (i, p) in placed.iter().enumerate() {
        stats[p.segment].times += 1;
        if !flags[i] {
            stats[p.segment].unaligned += 1;
        }
    }
    Diagnosis {
        keys: chart.key_count,
        notes: chart.notes.len(),
        times: times.len(),
        unaligned: flags.iter().filter(|&&a| !a).count(),
        segments: stats,
    }
}

/// Whether the chart's notes already sit on Etterna's rows.
#[must_use]
pub fn on_rows(chart: &RoxChart) -> bool {
    diagnose(chart).unaligned_pct() <= SNAP_THRESHOLD_PCT
}

/// One time per `chart.notes` entry, in µs, rebased so the first note is at 0.
///
/// `Rows::Quantized` keeps the times of a chart already on rows and snaps any
/// other chart onto Etterna's rows. `Rows::Exact` never snaps.
#[must_use]
pub fn rebased_times(chart: &RoxChart, rows: Rows) -> Vec<i64> {
    let first_us = chart.notes.iter().map(|n| n.time_us).min().unwrap_or(0);
    let snap = match rows {
        Rows::Exact => false,
        Rows::Quantized => !on_rows(chart),
    };
    if snap {
        snap_times(chart, first_us)
    } else {
        chart.notes.iter().map(|n| n.time_us - first_us).collect()
    }
}

/// A breakpoint of Etterna's BPM table: its row, its time in seconds, and the
/// BPM from there on.
#[derive(Debug, Clone, Copy)]
struct Breakpoint {
    row: i64,
    time_s: f64,
    bpm: f64,
}

/// Etterna's BPM table. The first BPM sits at row 0 and time 0. Each later
/// breakpoint takes its row from the table built so far (`MsToNoteRow`), so it
/// lands on the grid, and is dropped when it falls on row 0. A breakpoint on an
/// existing row replaces that row's BPM.
fn row_table(segs: &[Segment]) -> Vec<Breakpoint> {
    let first_bpm = segs.first().map_or(120.0, |s| s.bpm);
    let mut table = vec![Breakpoint {
        row: 0,
        time_s: 0.0,
        bpm: first_bpm,
    }];
    for seg in segs {
        let row = row_of_time(&table, seg.start_us as f64 / 1_000_000.0);
        if row == 0 {
            continue;
        }
        let time_s = time_of_row(&table, row);
        match table.last_mut() {
            Some(last) if last.row == row => last.bpm = seg.bpm,
            _ => table.push(Breakpoint {
                row,
                time_s,
                bpm: seg.bpm,
            }),
        }
    }
    table
}

/// Beats at `time_s` on the table, extrapolating the last segment past its end.
fn beat_at(table: &[Breakpoint], time_s: f64) -> f64 {
    let i = table
        .partition_point(|b| b.time_s <= time_s)
        .saturating_sub(1);
    let b = table[i];
    b.row as f64 / ROWS_PER_BEAT + (time_s - b.time_s) * b.bpm / 60.0
}

/// Etterna's `MsToNoteRow`: `lround(|beat| * 48)` of a time on the table.
fn row_of_time(table: &[Breakpoint], time_s: f64) -> i64 {
    (beat_at(table, time_s) * ROWS_PER_BEAT).abs().round() as i64
}

/// Time in seconds of a row on the table.
fn time_of_row(table: &[Breakpoint], row: i64) -> f64 {
    let i = table.partition_point(|b| b.row <= row).saturating_sub(1);
    let b = table[i];
    b.time_s + (row - b.row) as f64 * 60.0 / (ROWS_PER_BEAT * b.bpm)
}

/// Etterna's osu placement: each note's row comes from its time on the full
/// table, measured from the first note (`firstTap`). The table itself stays
/// anchored at chart time 0, as in Etterna's `MsToNoteRow`. Each note is then
/// timed from its row.
fn snap_times(chart: &RoxChart, first_us: i64) -> Vec<i64> {
    let table = row_table(&segments(chart));
    chart
        .notes
        .iter()
        .map(|n| {
            let row = row_of_time(&table, (n.time_us - first_us) as f64 / 1_000_000.0);
            (time_of_row(&table, row) * 1_000_000.0).round() as i64
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rox::Note;

    fn chart_at(bpm: f32, times: &[i64]) -> RoxChart {
        let mut chart = RoxChart::new(4);
        chart.timing_points.push(TimingPoint::Bpm {
            time_us: 0,
            bpm,
            signature: 4,
        });
        for (i, &t) in times.iter().enumerate() {
            chart.notes.push(Note::tap(t, (i % 4) as u8));
        }
        chart
    }

    /// Notes on the 120 BPM grid (row ≈ 10 416.67 µs) at ROX's rounding.
    fn grid_times(start: i64, count: i64) -> Vec<i64> {
        (0..count)
            .map(|k| start + (k as f64 * 10_416.666_666_7).round() as i64)
            .collect()
    }

    #[test]
    fn scattered_notes_snap_to_etterna_rows() {
        let chart = chart_at(120.0, &[0, 3_000, 7_000, 12_000, 19_000, 23_000, 31_000]);
        assert!(!on_rows(&chart));
        assert_eq!(
            rebased_times(&chart, Rows::Quantized),
            vec![0, 0, 10_417, 10_417, 20_833, 20_833, 31_250]
        );
    }

    #[test]
    fn notes_already_on_the_grid_keep_their_times() {
        let chart = chart_at(120.0, &grid_times(250_000, 24));
        assert!(on_rows(&chart));
        assert_eq!(
            rebased_times(&chart, Rows::Quantized),
            rebased_times(&chart, Rows::Exact)
        );
    }

    #[test]
    fn a_stop_shifts_the_grid_once_and_the_chart_stays_on_rows() {
        let mut times = grid_times(250_000, 12);
        times.extend(grid_times(250_000, 24)[12..].iter().map(|t| t + 1_234_567));
        let chart = chart_at(120.0, &times);
        assert!(on_rows(&chart));
        assert_eq!(
            rebased_times(&chart, Rows::Quantized),
            rebased_times(&chart, Rows::Exact)
        );
    }

    #[test]
    fn mines_are_not_evidence_for_or_against_the_grid() {
        let mut chart = chart_at(120.0, &grid_times(250_000, 24));
        for t in [1_111_111, 2_345_678, 3_456_789, 4_567_891, 5_678_912] {
            chart.notes.push(Note::mine(t, 3));
        }
        assert!(on_rows(&chart));
    }

    #[test]
    fn a_bpm_change_puts_its_breakpoint_on_a_row() {
        // 120 BPM to 1 s is exactly row 96; 240 BPM from there.
        let mut chart = chart_at(120.0, &[0, 1_005_000, 1_011_000]);
        chart.timing_points.push(TimingPoint::Bpm {
            time_us: 1_000_000,
            bpm: 240.0,
            signature: 4,
        });
        assert!(!on_rows(&chart));
        assert_eq!(
            rebased_times(&chart, Rows::Quantized),
            vec![0, 1_005_208, 1_010_417]
        );
    }

    #[test]
    fn inherited_points_scale_the_last_bpm_as_etterna_does() {
        let mut chart = chart_at(120.0, &[]);
        chart.timing_points.push(TimingPoint::Sv {
            time_us: 1_000_000,
            scroll_speed: 2.0,
        });
        let segs = segments(&chart);
        assert_eq!(segs.len(), 2);
        assert!((segs[1].bpm - 60.0).abs() < 1e-9);
    }
}
