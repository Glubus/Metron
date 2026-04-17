use std::cell::RefCell;
use super::super::smoothing::{SmoothMode, smooth_on_corners_into};
use super::super::note::Note;

thread_local! {
    static ABAR_DKS: RefCell<Vec<Vec<f64>>> = RefCell::new(Vec::new());
}

fn anchor_penalty(d_val: f64, dk0: f64, dk1: f64) -> f64 {
    let max_dk = dk0.max(dk1);
    if d_val < 0.02 {
        (0.75 + 0.5 * max_dk).min(1.0)
    } else if d_val < 0.07 {
        (0.65 + 5.0 * d_val + 0.5 * max_dk).min(1.0)
    } else {
        1.0
    }
}

fn build_dks_into(n: usize, k: usize, active_mask: &[u16], delta_ks: &[Vec<f64>], dks: &mut Vec<Vec<f64>>) {
    let num_pairs = k.saturating_sub(1);
    dks.resize_with(num_pairs, Vec::new);
    for col_dks in dks.iter_mut() {
        col_dks.resize(n, 0.0);
        col_dks[..n].fill(0.0);
    }
    for i in 0..n {
        let mut mask = active_mask[i];
        let mut prev_col: Option<usize> = None;
        while mask != 0 {
            let col = mask.trailing_zeros() as usize;
            if let Some(k0) = prev_col {
                let k1 = col;
                let dk0 = delta_ks[k0][i];
                let dk1 = delta_ks[k1][i];
                if k0 < dks.len() {
                    dks[k0][i] = (dk0 - dk1).abs() + 0.4 * ((dk0.max(dk1) - 0.11).max(0.0));
                }
            }
            prev_col = Some(col);
            mask &= mask - 1;
        }
    }
}

fn compute_a_step(a_corners: &[f64], base_corners: &[f64], active_mask: &[u16], dks: &[Vec<f64>], delta_ks: &[Vec<f64>]) -> Vec<f64> {
    let mut a_step = vec![1.0; a_corners.len()];
    for (i, &s) in a_corners.iter().enumerate() {
        let mut idx = base_corners.partition_point(|&v| v < s);
        if idx >= base_corners.len() { idx = base_corners.len() - 1; }
        let mut mask = active_mask[idx];
        let mut prev_col: Option<usize> = None;
        while mask != 0 {
            let col = mask.trailing_zeros() as usize;
            if let Some(k0) = prev_col {
                let k1 = col;
                if k0 < dks.len() {
                    let d_val = dks[k0][idx];
                    let dk0 = delta_ks[k0][idx];
                    let dk1 = delta_ks[k1][idx];
                    a_step[i] *= anchor_penalty(d_val, dk0, dk1);
                }
            }
            prev_col = Some(col);
            mask &= mask - 1;
        }
    }
    a_step
}

pub fn compute_abar(
    k: usize,
    _t: i64,
    _x: f64,
    _notes_by_column: &[Vec<Note>],
    active_mask: &[u16],
    delta_ks: &[Vec<f64>],
    a_corners: &[f64],
    base_corners: &[f64],
    out: &mut Vec<f64>,
) {
    let n = base_corners.len();
    ABAR_DKS.with(|dks_cell| {
        let mut dks = dks_cell.borrow_mut();
        build_dks_into(n, k, active_mask, delta_ks, &mut dks);
        let a_step = compute_a_step(a_corners, base_corners, active_mask, &dks, delta_ks);
        let na = a_corners.len();
        out.resize(na, 0.0);
        smooth_on_corners_into(a_corners, &a_step, 250.0, 1.0, SmoothMode::Avg, out);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_penalty_low_dval_is_below_one() {
        // d_val < 0.02: penalty = (0.75 + 0.5 * max_dk).min(1.0)
        // with max_dk = 0.1 → 0.75 + 0.05 = 0.80 < 1.0
        let p = anchor_penalty(0.01, 0.1, 0.05);
        assert!(p < 1.0, "expected penalty < 1.0 got {p}");
        assert!((p - 0.80).abs() < 1e-9);
    }

    #[test]
    fn test_anchor_penalty_mid_dval() {
        // d_val in [0.02, 0.07): (0.65 + 5*0.05 + 0.5*0.1).min(1.0) = 0.65+0.25+0.05 = 0.95
        let p = anchor_penalty(0.05, 0.1, 0.05);
        assert!((p - 0.95).abs() < 1e-9, "expected 0.95 got {p}");
    }

    #[test]
    fn test_anchor_penalty_high_dval_is_one() {
        assert_eq!(anchor_penalty(0.07, 0.1, 0.2), 1.0);
        assert_eq!(anchor_penalty(0.1, 0.5, 0.3), 1.0);
    }

    #[test]
    fn test_anchor_penalty_capped_at_one() {
        // 0.75 + 0.5 * 1.0 = 1.25, but capped at 1.0
        let p = anchor_penalty(0.01, 1.0, 1.0);
        assert_eq!(p, 1.0);
    }

    #[test]
    fn test_build_dks_empty_active_columns() {
        let n = 3;
        let k = 2;
        let active_mask: Vec<u16> = vec![0u16; n];
        let delta_ks = vec![vec![0.1; n], vec![0.2; n]];
        let mut dks = Vec::new();
        build_dks_into(n, k, &active_mask, &delta_ks, &mut dks);
        assert!(dks.iter().all(|col| col.iter().all(|&v| v == 0.0)));
    }

    #[test]
    fn test_build_dks_two_active_columns() {
        let n = 3;
        let k = 2;
        let active_mask: Vec<u16> = vec![0b11u16; n];
        let delta_ks = vec![vec![0.2; n], vec![0.1; n]];
        let mut dks = Vec::new();
        build_dks_into(n, k, &active_mask, &delta_ks, &mut dks);
        // k0=0, dk0=0.2, dk1=0.1: d = |0.2-0.1| + 0.4*(0.2-0.11) = 0.1 + 0.4*0.09 = 0.136
        for &v in &dks[0] {
            assert!((v - 0.136).abs() < 1e-9, "expected 0.136 got {v}");
        }
    }
}
