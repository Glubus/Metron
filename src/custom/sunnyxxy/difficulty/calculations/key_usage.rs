use std::cell::RefCell;
use super::super::note::Note;

thread_local! {
    // Flat buffers: index as [col * n + i]
    static KEY_USAGE_BUF: RefCell<Vec<bool>> = RefCell::new(Vec::new());
    static KEY_USAGE_400_BUF: RefCell<Vec<f64>> = RefCell::new(Vec::new());
}

/// Fills flat key_usage buffer (col-major: index = col * n + i) and calls `f` with it.
pub fn with_key_usage<R>(
    k: usize,
    t: i64,
    notes: &[Note],
    base_corners: &[f64],
    f: impl FnOnce(&[bool]) -> R,
) -> R {
    let n = base_corners.len();
    KEY_USAGE_BUF.with(|cell| {
        let mut buf = cell.borrow_mut();
        let total = k * n;
        if buf.len() < total { buf.resize(total, false); }
        // zero only the needed portion
        buf[..total].fill(false);
        for note in notes.iter() {
            let start_time = (note.hit_time - 150).max(0);
            let end_time = if note.tail_time < 0 {
                note.hit_time + 150
            } else {
                (note.tail_time + 150).min(t - 1)
            };
            let left_idx = base_corners.partition_point(|&v| v < start_time as f64);
            let right_idx = base_corners.partition_point(|&v| v < end_time as f64);
            let col_base = note.column * n;
            for i in left_idx..right_idx {
                buf[col_base + i] = true;
            }
        }
        f(&buf[..total])
    })
}

/// Fills flat key_usage_400 buffer (col-major: index = col * n + i) and calls `f` with it.
pub fn with_key_usage_400<R>(
    k: usize,
    t: i64,
    notes: &[Note],
    base_corners: &[f64],
    f: impl FnOnce(&[f64]) -> R,
) -> R {
    let n = base_corners.len();
    KEY_USAGE_400_BUF.with(|cell| {
        let mut buf = cell.borrow_mut();
        let total = k * n;
        if buf.len() < total { buf.resize(total, 0.0); }
        buf[..total].fill(0.0);
        for note in notes.iter() {
            let start_time = note.hit_time.max(0);
            let end_time = if note.tail_time < 0 {
                note.hit_time
            } else {
                (note.tail_time).min(t - 1)
            };
            let left400_idx = base_corners.partition_point(|&v| v < (start_time - 400) as f64);
            let left_idx = base_corners.partition_point(|&v| v < start_time as f64);
            let right_idx = base_corners.partition_point(|&v| v < end_time as f64);
            let right400_idx = base_corners.partition_point(|&v| v < (end_time + 400) as f64);
            let col_base = note.column * n;
            let usage = &mut buf[col_base..col_base + n];
            let base_val = 3.75 + ((end_time - start_time).min(1500) as f64) / 150.0;
            for i in left_idx..right_idx { usage[i] += base_val; }
            for i in left400_idx..left_idx {
                let diff = base_corners[i] - start_time as f64;
                usage[i] += 3.75 - 3.75 / (400.0 * 400.0) * diff * diff;
            }
            for i in right_idx..right400_idx {
                let diff = (base_corners[i] - end_time as f64).abs();
                usage[i] += 3.75 - 3.75 / (400.0 * 400.0) * diff * diff;
            }
        }
        f(&buf[..total])
    })
}
