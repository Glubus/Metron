use std::cell::RefCell;
use super::super::smoothing::{SmoothMode, smooth_on_corners_into};
use super::super::note::Note;

thread_local! {
    static JBAR_SMOOTH: RefCell<Vec<f64>> = RefCell::new(Vec::new());
    static JBAR_DEN: RefCell<Vec<f64>> = RefCell::new(Vec::new());
    static JBAR_J_COL: RefCell<Vec<f64>> = RefCell::new(Vec::new());
    static JBAR_DELTA_COL: RefCell<Vec<f64>> = RefCell::new(Vec::new());
}

fn jack_nerfer(delta: f64) -> f64 {
    let d = 0.15 + (delta - 0.08).abs();
    let d2 = d * d;
    1.0 - 7e-5 / (d2 * d2)
}

fn compute_column_jack_into(notes: &[Note], base_corners: &[f64], x_quarter: f64, j_col: &mut Vec<f64>, delta_col: &mut Vec<f64>) {
    let n = base_corners.len();
    j_col.resize(n, 0.0);
    j_col[..n].fill(0.0);
    delta_col.resize(n, 0.0);
    delta_col[..n].fill(1e9);
    if notes.len() < 2 { return; }
    let mut left_idx = 0usize;
    let mut right_idx = 0usize;
    for i in 0..notes.len() - 1 {
        let start = notes[i].hit_time as f64;
        let end = notes[i + 1].hit_time as f64;
        while left_idx < n && base_corners[left_idx] < start { left_idx += 1; }
        if right_idx < left_idx { right_idx = left_idx; }
        while right_idx < n && base_corners[right_idx] < end { right_idx += 1; }
        if left_idx >= right_idx { continue; }
        let delta = 0.001 * (end - start);
        let inv_delta = 1.0 / delta.max(1e-12);
        let val = inv_delta / (delta + 0.11 * x_quarter).max(1e-12);
        let j_val = val * jack_nerfer(delta);
        for idx in left_idx..right_idx { j_col[idx] = j_val; }
        for idx in left_idx..right_idx { delta_col[idx] = delta; }
    }
}

pub fn compute_jbar(k: usize, _t: i64, x: f64, notes_by_column: &[Vec<Note>], base_corners: &[f64], raw_delta: &mut Vec<Vec<f64>>, jbar_out: &mut Vec<f64>) {
    let x_quarter = x.sqrt().sqrt();
    let n = base_corners.len();
    raw_delta.resize_with(k, Vec::new);
    jbar_out.resize(n, 0.0);
    jbar_out[..n].fill(0.0);

    JBAR_J_COL.with(|jc_cell| {
        let mut j_col = jc_cell.borrow_mut();
        JBAR_DELTA_COL.with(|dc_cell| {
            let mut delta_col = dc_cell.borrow_mut();
            JBAR_SMOOTH.with(|s_cell| {
                let mut smooth_tmp = s_cell.borrow_mut();
                smooth_tmp.resize(n, 0.0);
                JBAR_DEN.with(|d_cell| {
                    let mut den_arr = d_cell.borrow_mut();
                    den_arr.resize(n, 0.0);
                    den_arr[..n].fill(0.0);

                    for col in 0..k {
                        compute_column_jack_into(&notes_by_column[col], base_corners, x_quarter, &mut j_col, &mut delta_col);
                        smooth_on_corners_into(base_corners, &j_col, 500.0, 0.001, SmoothMode::Sum, &mut smooth_tmp);
                        for i in 0..n {
                            let v = smooth_tmp[i].max(0.0);
                            let w = 1.0 / delta_col[i];
                            jbar_out[i] += v * v * v * v * v * w;
                            den_arr[i] += w;
                        }
                        let dk = &mut raw_delta[col];
                        dk.resize(n, 0.0);
                        dk.copy_from_slice(&delta_col[..n]);
                    }

                    for i in 0..n {
                        jbar_out[i] = (jbar_out[i] / den_arr[i].max(1e-9)).powf(0.2);
                    }
                });
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jack_nerfer_at_min_delta() {
        // delta=0.08: (delta - 0.08).abs() = 0, base = 0.15
        // nerfer = 1.0 - 7e-5 * 0.15^(-4) ≈ 1.0 - 7e-5 * 1975.3 ≈ 0.862
        let n = jack_nerfer(0.08);
        assert!((n - 0.862).abs() < 0.01, "expected ~0.862 got {n}");
        assert!(n < 1.0);
    }

    #[test]
    fn test_jack_nerfer_large_delta_approaches_one() {
        let n = jack_nerfer(10.0);
        assert!(n > 0.999, "expected close to 1.0 for large delta, got {n}");
    }

    #[test]
    fn test_jack_nerfer_always_positive() {
        for delta in [0.01, 0.05, 0.08, 0.1, 0.5, 1.0, 5.0] {
            let n = jack_nerfer(delta);
            assert!(n > 0.0, "expected positive nerfer for delta={delta}, got {n}");
        }
    }

    #[test]
    fn test_compute_column_jack_too_few_notes() {
        let corners = vec![0.0, 500.0, 1000.0];
        let notes_single = vec![Note::simple(0, 100)];
        let mut j = Vec::new();
        let mut d = Vec::new();
        compute_column_jack_into(&notes_single, &corners, 0.3, &mut j, &mut d);
        assert!(j.iter().all(|&v| v == 0.0));
        assert!(d.iter().all(|&v| v == 1e9));

        compute_column_jack_into(&[], &corners, 0.3, &mut j, &mut d);
        assert!(j.iter().all(|&v| v == 0.0));
        assert!(d.iter().all(|&v| v == 1e9));
    }

    #[test]
    fn test_compute_column_jack_two_notes_fills_range() {
        let notes = vec![Note::simple(0, 100), Note::simple(0, 300)];
        let corners: Vec<f64> = (0..500).map(|i| i as f64).collect();
        let mut j_col = Vec::new();
        let mut delta_col = Vec::new();
        compute_column_jack_into(&notes, &corners, 0.3, &mut j_col, &mut delta_col);
        // should have nonzero values in interval [100, 300)
        let has_jack = j_col[100..300].iter().any(|&v| v > 0.0);
        assert!(has_jack);
        let has_delta = delta_col[100..300].iter().any(|&v| v < 1e9);
        assert!(has_delta);
    }

    #[test]
    fn test_compute_jbar_empty_columns() {
        let corners: Vec<f64> = (0..5).map(|i| i as f64 * 100.0).collect();
        let notes_by_col: Vec<Vec<Note>> = vec![vec![], vec![]];
        let mut raw_delta = Vec::new();
        let mut jbar = Vec::new();
        compute_jbar(2, 0, 0.09, &notes_by_col, &corners, &mut raw_delta, &mut jbar);
        assert!(jbar.iter().all(|&v| v == 0.0), "empty notes → jbar all zero");
    }

    #[test]
    fn test_compute_jbar_positive_with_notes() {
        let corners: Vec<f64> = (0..500).map(|i| i as f64).collect();
        let notes: Vec<Note> = vec![Note::simple(0, 50), Note::simple(0, 150), Note::simple(0, 300)];
        let notes_by_col: Vec<Vec<Note>> = vec![notes, vec![]];
        let mut raw_delta = Vec::new();
        let mut jbar = Vec::new();
        compute_jbar(2, 0, 0.09, &notes_by_col, &corners, &mut raw_delta, &mut jbar);
        assert!(jbar.iter().any(|&v| v > 0.0), "should have nonzero jbar with notes");
    }
}
