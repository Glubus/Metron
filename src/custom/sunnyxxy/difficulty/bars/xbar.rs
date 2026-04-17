use std::cell::RefCell;
use super::super::smoothing::{smooth_on_corners_into, SmoothMode};
use super::super::note::Note;

thread_local! {
    static XBAR_X_KS: RefCell<Vec<f64>> = RefCell::new(Vec::new());
    static XBAR_FAST_CROSS: RefCell<Vec<f64>> = RefCell::new(Vec::new());
    static XBAR_BASE: RefCell<Vec<f64>> = RefCell::new(Vec::new());
    static XBAR_PAIR_TIMES: RefCell<Vec<f64>> = RefCell::new(Vec::new());
}

// Stack-allocated coefficient arrays (max k=10 → 11 elements)
const MAX_COLS: usize = 11;

fn cross_coefficients(k: usize, out: &mut [f64; MAX_COLS]) -> usize {
    let len = k + 1;
    match k {
        0 => out[..1].copy_from_slice(&[-1.0]),
        1 => out[..2].copy_from_slice(&[0.075, 0.075]),
        2 => out[..3].copy_from_slice(&[0.125, 0.05, 0.125]),
        3 => out[..4].copy_from_slice(&[0.125, 0.125, 0.125, 0.125]),
        4 => out[..5].copy_from_slice(&[0.175, 0.25, 0.05, 0.25, 0.175]),
        5 => out[..6].copy_from_slice(&[0.175, 0.25, 0.175, 0.175, 0.25, 0.175]),
        6 => out[..7].copy_from_slice(&[0.225, 0.35, 0.25, 0.05, 0.25, 0.35, 0.225]),
        7 => out[..8].copy_from_slice(&[0.225, 0.35, 0.25, 0.225, 0.225, 0.25, 0.35, 0.225]),
        8 => out[..9].copy_from_slice(&[0.275, 0.45, 0.35, 0.25, 0.05, 0.25, 0.35, 0.45, 0.275]),
        9 => out[..10].copy_from_slice(&[0.275, 0.45, 0.35, 0.25, 0.275, 0.275, 0.25, 0.35, 0.45, 0.275]),
        10 => out[..11].copy_from_slice(&[0.325, 0.55, 0.45, 0.35, 0.25, 0.05, 0.25, 0.35, 0.45, 0.55, 0.325]),
        _ => panic!("unsupported key count {k}"),
    }
    len
}

fn collect_pair_times_into(a: &[Note], b: &[Note], times: &mut Vec<f64>) {
    times.clear();
    let (mut ia, mut ib) = (0, 0);
    while ia < a.len() || ib < b.len() {
        if ia < a.len() && (ib >= b.len() || a[ia].hit_time <= b[ib].hit_time) {
            times.push(a[ia].hit_time as f64); ia += 1;
        } else {
            times.push(b[ib].hit_time as f64); ib += 1;
        }
    }
}

fn fill_interval(x_col: &mut [f64], fc_col: &mut [f64], idx_s: usize, idx_e: usize, prev: f64, next: f64, x: f64, cross_comp: f64, col: usize, active_mask: &[u16]) {
    let delta = 0.001 * (next - prev);
    let inv = 1.0 / x.max(delta);
    let mut val = 0.16 * inv * inv;
    let pc_bit = if col > 0 { 1u16 << (col - 1) } else { 0u16 };
    let col_bit = 1u16 << col;
    let mask_s = active_mask.get(idx_s).copied().unwrap_or(0);
    let mask_e = active_mask.get(idx_e).copied().unwrap_or(0);
    let inactive_prev = (mask_s & pc_bit == 0) && (mask_e & pc_bit == 0);
    let inactive_cur = (mask_s & col_bit == 0) && (mask_e & col_bit == 0);
    if inactive_prev || inactive_cur { val *= cross_comp; }
    let d = delta.max(0.06).max(0.75 * x);
    let fc_val = (0.4 / (d * d) - 80.0).max(0.0);
    x_col[idx_s..idx_e].fill(val);
    fc_col[idx_s..idx_e].fill(fc_val);
}

fn fill_col_contributions_into(times: &[f64], base_corners: &[f64], x: f64, cross_comp: f64, col: usize, active_mask: &[u16], x_col: &mut [f64], fc_col: &mut [f64]) {
    let n = base_corners.len();
    x_col[..n].fill(0.0);
    fc_col[..n].fill(0.0);
    if times.len() < 2 { return; }
    let mut idx_s = 0usize;
    let mut idx_e = 0usize;
    for w in times.windows(2) {
        let (prev_time, next_time) = (w[0], w[1]);
        while idx_s < n && base_corners[idx_s] < prev_time { idx_s += 1; }
        if idx_e < idx_s { idx_e = idx_s; }
        while idx_e < n && base_corners[idx_e] < next_time { idx_e += 1; }
        if idx_s >= idx_e { continue; }
        fill_interval(x_col, fc_col, idx_s, idx_e, prev_time, next_time, x, cross_comp, col, active_mask);
    }
}

// x_ks and fast_cross are flat col-major: index = col * n + i
// Iterate col-outer, i-inner for sequential cache-friendly access within each col's slice
fn merge_xbar_contributions_into(x_ks: &[f64], fast_cross: &[f64], cross_coeff: &[f64], n: usize, k: usize, out: &mut [f64]) {
    out[..n].fill(0.0);
    for col in 0..=k {
        let coeff = cross_coeff[col];
        let xk_col = &x_ks[col * n..(col + 1) * n];
        for i in 0..n {
            out[i] += xk_col[i] * coeff;
        }
    }
    for col in 0..k {
        let c0 = cross_coeff[col];
        let c1 = cross_coeff[col + 1];
        let fc0 = &fast_cross[col * n..(col + 1) * n];
        let fc1 = &fast_cross[(col + 1) * n..(col + 2) * n];
        for i in 0..n {
            out[i] += (fc0[i] * c0 * fc1[i] * c1).sqrt();
        }
    }
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
    active_mask: &[u16],
    base_corners: &[f64],
    out: &mut Vec<f64>,
) {
    let mut cross_coeff_arr = [0.0f64; MAX_COLS];
    let len = cross_coefficients(k, &mut cross_coeff_arr);
    let cross_coeff = &cross_coeff_arr[..len];
    let mut cross_comp_arr = [0.0f64; MAX_COLS];
    for i in 0..len { cross_comp_arr[i] = 1.0 - cross_coeff[i]; }
    let cross_comp = &cross_comp_arr[..len];
    let n = base_corners.len();
    let total_cols = k + 1;
    XBAR_X_KS.with(|xk_cell| {
        let mut x_ks = xk_cell.borrow_mut();
        XBAR_FAST_CROSS.with(|fc_cell| {
            let mut fast_cross = fc_cell.borrow_mut();
            XBAR_BASE.with(|xb_cell| {
                let mut x_base = xb_cell.borrow_mut();
                XBAR_PAIR_TIMES.with(|pt_cell| {
                    let mut pair_times = pt_cell.borrow_mut();
                    let flat_len = total_cols * n;
                    x_ks.resize(flat_len, 0.0);
                    fast_cross.resize(flat_len, 0.0);
                    x_base.resize(n, 0.0);
                    for col in 0..=k {
                        let (a, b) = column_notes(col, k, notes_by_column);
                        collect_pair_times_into(a, b, &mut pair_times);
                        let (xk_slice, fc_slice) = (&mut x_ks[col*n..(col+1)*n], &mut fast_cross[col*n..(col+1)*n]);
                        fill_col_contributions_into(&pair_times, base_corners, x, cross_comp[col], col, active_mask, xk_slice, fc_slice);
                    }
                    merge_xbar_contributions_into(&x_ks, &fast_cross, cross_coeff, n, k, &mut x_base);
                    out.resize(n, 0.0);
                    smooth_on_corners_into(base_corners, &x_base, 500.0, 0.001, SmoothMode::Sum, out);
                })
            })
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coeffs(k: usize) -> Vec<f64> {
        let mut arr = [0.0f64; MAX_COLS];
        let len = cross_coefficients(k, &mut arr);
        arr[..len].to_vec()
    }

    #[test]
    fn test_cross_coefficients_k4() {
        assert_eq!(coeffs(4), vec![0.175, 0.25, 0.05, 0.25, 0.175]);
    }

    #[test]
    fn test_cross_coefficients_k1() {
        assert_eq!(coeffs(1), vec![0.075, 0.075]);
    }

    #[test]
    fn test_cross_coefficients_length() {
        for k in 1..=10 {
            assert_eq!(coeffs(k).len(), k + 1, "expected len {} for k={}", k + 1, k);
        }
    }

    #[test]
    fn test_collect_pair_times_empty() {
        let mut times = Vec::new();
        collect_pair_times_into(&[], &[], &mut times);
        assert!(times.is_empty());
    }

    #[test]
    fn test_collect_pair_times_merges_sorted() {
        let a = vec![Note::simple(0, 10), Note::simple(0, 30)];
        let b = vec![Note::simple(1, 20), Note::simple(1, 40)];
        let mut times = Vec::new();
        collect_pair_times_into(&a, &b, &mut times);
        assert_eq!(times, vec![10.0, 20.0, 30.0, 40.0]);
    }

    #[test]
    fn test_collect_pair_times_one_empty() {
        let a = vec![Note::simple(0, 100), Note::simple(0, 200)];
        let mut times = Vec::new();
        collect_pair_times_into(&a, &[], &mut times);
        assert_eq!(times, vec![100.0, 200.0]);
    }

    #[test]
    fn test_merge_xbar_contributions_zeros() {
        let n = 3;
        let k = 2;
        let x_ks = vec![0.0f64; (k + 1) * n];
        let fast_cross = vec![0.0f64; (k + 1) * n];
        let c = coeffs(k);
        let mut result = vec![0.0f64; n];
        merge_xbar_contributions_into(&x_ks, &fast_cross, &c, n, k, &mut result);
        assert!(result.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_merge_xbar_contributions_nonzero() {
        let n = 2;
        let k = 1;
        let c = coeffs(k);
        let x_ks = vec![1.0f64; (k + 1) * n];
        let fast_cross = vec![0.0f64; (k + 1) * n];
        let mut result = vec![0.0f64; n];
        merge_xbar_contributions_into(&x_ks, &fast_cross, &c, n, k, &mut result);
        // sum1 = 1.0 * 0.075 + 1.0 * 0.075 = 0.15 per element
        assert!((result[0] - 0.15).abs() < 1e-9);
    }
}
