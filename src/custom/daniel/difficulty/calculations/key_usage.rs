use super::super::interpolation::search_left;
use super::super::note::Note;
use std::cell::RefCell;

thread_local! {
    static KEY_USAGE_BUF: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
    static KEY_USAGE_400_BUF: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
}

pub fn with_key_usage<R>(
    key_count: usize,
    total_duration: i64,
    notes: &[Note],
    base_corners: &[f64],
    f: impl FnOnce(&[bool]) -> R,
) -> R {
    let corner_count = base_corners.len();
    KEY_USAGE_BUF.with(|cell| {
        let mut buf = cell.borrow_mut();
        let total = key_count * corner_count;
        if buf.len() < total {
            buf.resize(total, false);
        }
        buf[..total].fill(false);

        for note in notes {
            let start = (note.hit_time - 150).max(0) as f64;
            let end = (note.hit_time + 150).min(total_duration - 1) as f64;
            let left_index = search_left(base_corners, start);
            let right_index = search_left(base_corners, end);
            let column_base = note.column * corner_count;
            for index in left_index..right_index {
                buf[column_base + index] = true;
            }
        }

        f(&buf[..total])
    })
}

pub fn with_key_usage_400<R>(
    key_count: usize,
    notes: &[Note],
    base_corners: &[f64],
    f: impl FnOnce(&[f64]) -> R,
) -> R {
    let corner_count = base_corners.len();
    KEY_USAGE_400_BUF.with(|cell| {
        let mut buf = cell.borrow_mut();
        let total = key_count * corner_count;
        if buf.len() < total {
            buf.resize(total, 0.0);
        }
        buf[..total].fill(0.0);

        for note in notes {
            let start = note.hit_time as f64;
            let left_index = search_left(base_corners, start - 400.0);
            let right_index = search_left(base_corners, start + 400.0);
            let mid_index = search_left(base_corners, start);
            let usage = &mut buf[note.column * corner_count..(note.column + 1) * corner_count];

            usage[mid_index] += 3.75;
            for index in left_index..mid_index {
                let delta = base_corners[index] - start;
                usage[index] += 3.75 - 3.75 / 400.0_f64.powi(2) * delta * delta;
            }
            for index in (mid_index + 1)..right_index {
                let delta = base_corners[index] - start;
                usage[index] += 3.75 - 3.75 / 400.0_f64.powi(2) * delta * delta;
            }
        }

        f(&buf[..total])
    })
}

pub fn compute_active_masks(key_count: usize, key_usage: &[bool], corner_count: usize) -> Vec<u16> {
    let mut active_masks = vec![0u16; corner_count];
    for column in 0..key_count {
        let bit = 1u16 << column;
        let column_base = column * corner_count;
        for index in 0..corner_count {
            if key_usage[column_base + index] {
                active_masks[index] |= bit;
            }
        }
    }
    active_masks
}
