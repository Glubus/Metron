use super::super::smoothing::smooth_on_corners;
use super::super::smoothing::SmoothMode;
use super::super::note::Note;

fn cross_coefficients(k: usize) -> Vec<f64> {
    match k {
        0 => vec![-1.0],
        1 => vec![0.075, 0.075],
        2 => vec![0.125, 0.05, 0.125],
        3 => vec![0.125, 0.125, 0.125, 0.125],
        4 => vec![0.175, 0.25, 0.05, 0.25, 0.175],
        5 => vec![0.175, 0.25, 0.175, 0.175, 0.25, 0.175],
        6 => vec![0.225, 0.35, 0.25, 0.05, 0.25, 0.35, 0.225],
        7 => vec![0.225, 0.35, 0.25, 0.225, 0.225, 0.25, 0.35, 0.225],
        8 => vec![0.275, 0.45, 0.35, 0.25, 0.05, 0.25, 0.35, 0.45, 0.275],
        9 => vec![0.275, 0.45, 0.35, 0.25, 0.275, 0.275, 0.25, 0.35, 0.45, 0.275],
        10 => vec![0.325, 0.55, 0.45, 0.35, 0.25, 0.05, 0.25, 0.35, 0.45, 0.55, 0.325],
        _ => panic!("unsupported key count {k}"),
    }
}

fn collect_pair_times(a: &[Note], b: &[Note]) -> Vec<f64> {
    let mut times = Vec::with_capacity(a.len() + b.len());
    let (mut ia, mut ib) = (0, 0);
    while ia < a.len() || ib < b.len() {
        if ia < a.len() && (ib >= b.len() || a[ia].hit_time <= b[ib].hit_time) {
            times.push(a[ia].hit_time as f64); ia += 1;
        } else {
            times.push(b[ib].hit_time as f64); ib += 1;
        }
    }
    times
}

fn fill_interval(x_col: &mut [f64], fc_col: &mut [f64], idx_s: usize, idx_e: usize, prev: f64, next: f64, x: f64, cross_comp: f64, col: usize, active_columns: &[Vec<usize>]) {
    let delta = 0.001 * (next - prev);
    let inv = 1.0 / x.max(delta);
    let mut val = 0.16 * inv * inv;
    let pc = col.wrapping_sub(1);
    let inactive_prev = !active_columns.get(idx_s).map_or(false, |v| v.contains(&pc))
        && !active_columns.get(idx_e).map_or(false, |v| v.contains(&pc));
    let inactive_cur = !active_columns.get(idx_s).map_or(false, |v| v.contains(&col))
        && !active_columns.get(idx_e).map_or(false, |v| v.contains(&col));
    if inactive_prev || inactive_cur { val *= cross_comp; }
    let base = delta.max(0.06).max(0.75 * x).powf(-2.0);
    for idx in idx_s..idx_e { x_col[idx] = val; fc_col[idx] = (0.4 * base - 80.0).max(0.0); }
}

fn fill_col_contributions(times: &[f64], base_corners: &[f64], x: f64, cross_comp: f64, col: usize, active_columns: &[Vec<usize>]) -> (Vec<f64>, Vec<f64>) {
    let n = base_corners.len();
    let mut x_col = vec![0.0; n];
    let mut fc_col = vec![0.0; n];
    if times.len() < 2 { return (x_col, fc_col); }
    let mut idx_s = 0usize;
    let mut idx_e = 0usize;
    for w in times.windows(2) {
        let (prev_time, next_time) = (w[0], w[1]);
        while idx_s < n && base_corners[idx_s] < prev_time { idx_s += 1; }
        if idx_e < idx_s { idx_e = idx_s; }
        while idx_e < n && base_corners[idx_e] < next_time { idx_e += 1; }
        if idx_s >= idx_e { continue; }
        fill_interval(&mut x_col, &mut fc_col, idx_s, idx_e, prev_time, next_time, x, cross_comp, col, active_columns);
    }
    (x_col, fc_col)
}

fn merge_xbar_contributions(x_ks: &[Vec<f64>], fast_cross: &[Vec<f64>], cross_coeff: &[f64], n: usize, k: usize) -> Vec<f64> {
    (0..n).map(|i| {
        let sum1: f64 = (0..=k).map(|col| x_ks[col][i] * cross_coeff[col]).sum();
        let sum2: f64 = (0..k).map(|col| {
            (fast_cross[col][i] * cross_coeff[col] * fast_cross[col + 1][i] * cross_coeff[col + 1]).sqrt()
        }).sum();
        sum1 + sum2
    }).collect()
}

fn column_notes<'a>(col: usize, k: usize, notes_by_column: &'a [Vec<Note>]) -> (&'a [Note], &'a [Note]) {
    if col == 0 { (&notes_by_column[0], &[]) }
    else if col == k { (&notes_by_column[k - 1], &[]) }
    else { (&notes_by_column[col - 1], &notes_by_column[col]) }
}

pub fn compute_xbar(
    k: usize,
    _t: i64,
    x: f64,
    notes_by_column: &[Vec<Note>],
    active_columns: &[Vec<usize>],
    base_corners: &[f64],
) -> Vec<f64> {
    let cross_coeff = cross_coefficients(k);
    let cross_comp: Vec<f64> = cross_coeff.iter().map(|&c| 1.0 - c).collect();
    let n = base_corners.len();
    let mut x_ks: Vec<Vec<f64>> = vec![vec![0.0; n]; k + 1];
    let mut fast_cross: Vec<Vec<f64>> = vec![vec![0.0; n]; k + 1];
    for col in 0..=k {
        let (a, b) = column_notes(col, k, notes_by_column);
        let times = collect_pair_times(a, b);
        let (xc, fc) = fill_col_contributions(&times, base_corners, x, cross_comp[col], col, active_columns);
        x_ks[col] = xc;
        fast_cross[col] = fc;
    }
    let x_base = merge_xbar_contributions(&x_ks, &fast_cross, &cross_coeff, n, k);
    smooth_on_corners(base_corners, &x_base, 500.0, 0.001, SmoothMode::Sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_coefficients_k4() {
        let c = cross_coefficients(4);
        assert_eq!(c, vec![0.175, 0.25, 0.05, 0.25, 0.175]);
    }

    #[test]
    fn test_cross_coefficients_k1() {
        let c = cross_coefficients(1);
        assert_eq!(c, vec![0.075, 0.075]);
    }

    #[test]
    fn test_cross_coefficients_length() {
        for k in 1..=10 {
            let c = cross_coefficients(k);
            assert_eq!(c.len(), k + 1, "expected len {} for k={}", k + 1, k);
        }
    }

    #[test]
    fn test_collect_pair_times_empty() {
        let times = collect_pair_times(&[], &[]);
        assert!(times.is_empty());
    }

    #[test]
    fn test_collect_pair_times_merges_sorted() {
        let a = vec![Note::simple(0, 10), Note::simple(0, 30)];
        let b = vec![Note::simple(1, 20), Note::simple(1, 40)];
        let times = collect_pair_times(&a, &b);
        assert_eq!(times, vec![10.0, 20.0, 30.0, 40.0]);
    }

    #[test]
    fn test_collect_pair_times_one_empty() {
        let a = vec![Note::simple(0, 100), Note::simple(0, 200)];
        let times = collect_pair_times(&a, &[]);
        assert_eq!(times, vec![100.0, 200.0]);
    }

    #[test]
    fn test_merge_xbar_contributions_zeros() {
        let n = 3;
        let k = 2;
        let x_ks = vec![vec![0.0; n]; k + 1];
        let fast_cross = vec![vec![0.0; n]; k + 1];
        let cross_coeff = cross_coefficients(k);
        let result = merge_xbar_contributions(&x_ks, &fast_cross, &cross_coeff, n, k);
        assert!(result.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_merge_xbar_contributions_nonzero() {
        let n = 2;
        let k = 1;
        // k=1: coeff = [0.075, 0.075], 2 cols (0 and 1)
        let cross_coeff = cross_coefficients(k);
        let mut x_ks = vec![vec![0.0; n]; k + 1];
        let fast_cross = vec![vec![0.0; n]; k + 1];
        x_ks[0] = vec![1.0, 1.0];
        x_ks[1] = vec![1.0, 1.0];
        let result = merge_xbar_contributions(&x_ks, &fast_cross, &cross_coeff, n, k);
        // sum1 = 1.0 * 0.075 + 1.0 * 0.075 = 0.15 per element
        assert!((result[0] - 0.15).abs() < 1e-9);
    }
}
