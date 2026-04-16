use super::super::smoothing::{SmoothMode, smooth_on_corners};
use super::super::note::Note;
use std::collections::HashMap;

fn jack_nerfer(delta: f64) -> f64 {
    1.0 - 7e-5 * (0.15 + (delta - 0.08).abs()).powf(-4.0)
}

fn compute_column_jack(notes: &[Note], base_corners: &[f64], x_quarter: f64) -> (Vec<f64>, Vec<f64>) {
    let n = base_corners.len();
    let mut j_col = vec![0.0; n];
    let mut delta_col = vec![1e9; n];
    if notes.len() < 2 { return (j_col, delta_col); }
    let mut left_idx = 0usize;
    let mut right_idx = 0usize;
    for i in 0..notes.len() - 1 {
        let start = notes[i].hit_time as f64;
        let end = notes[i + 1].hit_time as f64;
        while left_idx < base_corners.len() && base_corners[left_idx] < start { left_idx += 1; }
        if right_idx < left_idx { right_idx = left_idx; }
        while right_idx < base_corners.len() && base_corners[right_idx] < end { right_idx += 1; }
        if left_idx >= right_idx { continue; }
        let delta = 0.001 * (end - start);
        let inv_delta = 1.0 / delta.max(1e-12);
        let val = inv_delta / (delta + 0.11 * x_quarter).max(1e-12);
        let j_val = val * jack_nerfer(delta);
        for idx in left_idx..right_idx { j_col[idx] = j_val; }
        for idx in left_idx..right_idx { delta_col[idx] = delta; }
    }
    (j_col, delta_col)
}

fn aggregate_jack_columns(jbar_ks: &[Vec<f64>], delta_ks: &[Vec<f64>], n: usize, k: usize) -> Vec<f64> {
    let mut jbar = vec![0.0; n];
    for i in 0..n {
        let mut num = 0.0;
        let mut den = 0.0;
        for col in 0..k {
            let v = jbar_ks[col][i];
            let w = 1.0 / delta_ks[col][i];
            num += v.max(0.0).powf(5.0) * w;
            den += w;
        }
        jbar[i] = (num / den.max(1e-9)).powf(0.2);
    }
    jbar
}

pub fn compute_jbar(k: usize, _t: i64, x: f64, notes_by_column: &[Vec<Note>], base_corners: &[f64])
    -> (HashMap<usize, Vec<f64>>, Vec<f64>)
{
    let x_quarter = x.powf(0.25);
    let n = base_corners.len();
    let (raw_j, raw_delta): (Vec<_>, Vec<_>) = (0..k)
        .map(|col| compute_column_jack(&notes_by_column[col], base_corners, x_quarter))
        .unzip();
    let jbar_ks: Vec<Vec<f64>> = raw_j.iter()
        .map(|j_col| smooth_on_corners(base_corners, j_col, 500.0, 0.001, SmoothMode::Sum))
        .collect();
    let jbar = aggregate_jack_columns(&jbar_ks, &raw_delta, n, k);
    let delta_ks_map: HashMap<usize, Vec<f64>> = raw_delta.into_iter().enumerate().collect();
    (delta_ks_map, jbar)
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
        let (j, d) = compute_column_jack(&notes_single, &corners, 0.3);
        assert!(j.iter().all(|&v| v == 0.0));
        assert!(d.iter().all(|&v| v == 1e9));

        let (j2, d2) = compute_column_jack(&[], &corners, 0.3);
        assert!(j2.iter().all(|&v| v == 0.0));
        assert!(d2.iter().all(|&v| v == 1e9));
    }

    #[test]
    fn test_compute_column_jack_two_notes_fills_range() {
        let notes = vec![Note::simple(0, 100), Note::simple(0, 300)];
        let corners: Vec<f64> = (0..500).map(|i| i as f64).collect();
        let (j_col, delta_col) = compute_column_jack(&notes, &corners, 0.3);
        // should have nonzero values in interval [100, 300)
        let has_jack = j_col[100..300].iter().any(|&v| v > 0.0);
        assert!(has_jack);
        let has_delta = delta_col[100..300].iter().any(|&v| v < 1e9);
        assert!(has_delta);
    }

    #[test]
    fn test_aggregate_jack_columns_uniform() {
        let n = 5;
        let k = 2;
        // both columns with same jbar values
        let jbar_ks = vec![vec![2.0; n], vec![2.0; n]];
        // delta_ks with values that give equal weights (delta=1.0 → w=1.0)
        let delta_ks = vec![vec![1.0; n], vec![1.0; n]];
        let jbar = aggregate_jack_columns(&jbar_ks, &delta_ks, n, k);
        // both cols same → result = (2^5 + 2^5) / (1+1))^0.2 = (2*32/2)^0.2 = 32^0.2 = 2.0
        for &v in &jbar {
            assert!((v - 2.0).abs() < 1e-9, "expected 2.0 got {v}");
        }
    }

    #[test]
    fn test_aggregate_jack_columns_zero_input() {
        let n = 3;
        let k = 2;
        let jbar_ks = vec![vec![0.0; n], vec![0.0; n]];
        let delta_ks = vec![vec![1.0; n], vec![1.0; n]];
        let jbar = aggregate_jack_columns(&jbar_ks, &delta_ks, n, k);
        assert!(jbar.iter().all(|&v| v == 0.0));
    }
}
