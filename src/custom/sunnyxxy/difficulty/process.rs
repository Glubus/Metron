use super::interpolation::{interp_values, step_interp};
use super::smoothing::rescale_high;
use super::map_data::MapData;
use super::note::Note;
use super::bars::abar::compute_abar;
use super::bars::jbar::compute_jbar;
use super::bars::pbar::compute_pbar;
use super::bars::rbar::compute_rbar;
use super::bars::xbar::compute_xbar;
use super::calculations::anchor::compute_anchor;
use super::calculations::ck::compute_c_and_ks;
use super::calculations::corners::get_corners;
use super::calculations::key_usage::{get_key_usage, get_key_usage_400};
use super::calculations::ln::ln_bodies_count_sparse_representation;

pub fn calculate_internal(map_data: &MapData) -> f64 {
    let (all_corners, base_corners, a_corners, active_columns, anchor) = phase1(map_data);
    let (jbar, xbar, pbar, abar, rbar, c_arr, ks_arr) = phase2(
        map_data, &active_columns, &a_corners, &base_corners, &all_corners, &anchor,
    );
    let (_s_all, _t_all, d_all) = phase3(&jbar, &xbar, &pbar, &abar, &rbar, &c_arr, &ks_arr);
    let (percentile_93, percentile_83, weighted_mean) = phase4(&d_all, &c_arr, &all_corners);
    phase5(percentile_93, percentile_83, weighted_mean, &map_data.notes, &map_data.long_notes)
}

fn compute_active_columns_from_map(map_data: &MapData, base_corners: &[f64]) -> Vec<Vec<usize>> {
    let key_usage = get_key_usage(map_data.column_count, map_data.total_duration, &map_data.notes, base_corners);
    compute_active_columns(&key_usage, map_data.column_count, base_corners.len())
}

fn compute_anchor_from_map(map_data: &MapData, base_corners: &[f64]) -> Vec<f64> {
    let key_usage_400 = get_key_usage_400(map_data.column_count, map_data.total_duration, &map_data.notes, base_corners);
    compute_anchor(map_data.column_count, &key_usage_400, base_corners)
}

fn phase1(map_data: &MapData) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<Vec<usize>>, Vec<f64>) {
    let (all_corners, base_corners, a_corners) = get_corners(map_data.total_duration, &map_data.notes);
    let active_columns = compute_active_columns_from_map(map_data, &base_corners);
    let anchor = compute_anchor_from_map(map_data, &base_corners);
    (all_corners, base_corners, a_corners, active_columns, anchor)
}

fn compute_jbar_step(map_data: &MapData, base_corners: &[f64], all_corners: &[f64]) -> (std::collections::HashMap<usize, Vec<f64>>, Vec<f64>) {
    let (delta_ks, jbar) = compute_jbar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes_by_column, base_corners);
    (delta_ks, interp_values(all_corners, base_corners, &jbar))
}

fn compute_xbar_step(map_data: &MapData, active_columns: &[Vec<usize>], base_corners: &[f64], all_corners: &[f64]) -> Vec<f64> {
    let xbar = compute_xbar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes_by_column, active_columns, base_corners);
    interp_values(all_corners, base_corners, &xbar)
}

fn compute_pbar_step(map_data: &MapData, anchor: &[f64], base_corners: &[f64], all_corners: &[f64]) -> Vec<f64> {
    let ln_rep = ln_bodies_count_sparse_representation(&map_data.long_notes, map_data.total_duration);
    let pbar = compute_pbar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes, &ln_rep, anchor, base_corners);
    interp_values(all_corners, base_corners, &pbar)
}

fn compute_abar_step(map_data: &MapData, active_columns: &[Vec<usize>], delta_ks: &std::collections::HashMap<usize, Vec<f64>>, a_corners: &[f64], base_corners: &[f64], all_corners: &[f64]) -> Vec<f64> {
    let abar = compute_abar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes_by_column, active_columns, delta_ks, a_corners, base_corners);
    interp_values(all_corners, a_corners, &abar)
}

fn compute_rbar_step(map_data: &MapData, base_corners: &[f64], all_corners: &[f64]) -> Vec<f64> {
    let rbar = compute_rbar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes_by_column, &map_data.tail_sequence, base_corners);
    interp_values(all_corners, base_corners, &rbar)
}

fn compute_c_ks_step(map_data: &MapData, base_corners: &[f64], all_corners: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let key_usage = get_key_usage(map_data.column_count, map_data.total_duration, &map_data.notes, base_corners);
    let (c_step, ks_step) = compute_c_and_ks(map_data.column_count, map_data.total_duration, &map_data.notes, &key_usage, base_corners);
    (step_interp(all_corners, base_corners, &c_step), step_interp(all_corners, base_corners, &ks_step))
}

fn phase2(map_data: &MapData, active_columns: &[Vec<usize>], a_corners: &[f64], base_corners: &[f64], all_corners: &[f64], anchor: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let (delta_ks, jbar) = compute_jbar_step(map_data, base_corners, all_corners);
    let xbar = compute_xbar_step(map_data, active_columns, base_corners, all_corners);
    let pbar = compute_pbar_step(map_data, anchor, base_corners, all_corners);
    let abar = compute_abar_step(map_data, active_columns, &delta_ks, a_corners, base_corners, all_corners);
    let rbar = compute_rbar_step(map_data, base_corners, all_corners);
    let (c_arr, ks_arr) = compute_c_ks_step(map_data, base_corners, all_corners);
    (jbar, xbar, pbar, abar, rbar, c_arr, ks_arr)
}

fn compute_s(j: f64, p: f64, a: f64, r: f64, c: f64, ks: f64) -> f64 {
    let jack_term = a.powf(3.0 / ks) * j.min(8.0 + 0.85 * j);
    let stream_term = a.powf(2.0 / 3.0) * (0.8 * p + r * 35.0 / (c + 8.0));
    ((0.4 * jack_term.powf(1.5)) + (0.6 * stream_term.powf(1.5))).powf(2.0 / 3.0)
}

fn compute_t(s: f64, x: f64, a: f64, ks: f64) -> f64 {
    (a.powf(3.0 / ks) * x) / (x + s + 1.0)
}

fn compute_d(s: f64, t: f64) -> f64 {
    2.7 * s.sqrt() * t.powf(1.5) + s * 0.27
}

fn phase3(jbar: &[f64], xbar: &[f64], pbar: &[f64], abar: &[f64], rbar: &[f64], c_arr: &[f64], ks_arr: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let s_all: Vec<f64> = jbar.iter().zip(xbar).zip(pbar).zip(abar).zip(rbar).zip(c_arr).zip(ks_arr)
        .map(|((((((&j, &_x), &p), &a), &r), &c), &ks)| compute_s(j, p, a, r, c, ks))
        .collect();
    let t_all: Vec<f64> = s_all.iter().zip(xbar).zip(abar).zip(ks_arr)
        .map(|(((&s, &x), &a), &ks)| compute_t(s, x, a, ks))
        .collect();
    let d_all: Vec<f64> = s_all.iter().zip(t_all.iter())
        .map(|(&s, &t)| compute_d(s, t))
        .collect();
    (s_all, t_all, d_all)
}

fn cumulative_weights(weights: &[f64]) -> Vec<f64> {
    weights.iter().scan(0.0, |acc, &w| { *acc += w; Some(*acc) }).collect()
}

fn percentile_pair(d_sorted: &[f64], pct_indices: &[usize]) -> (f64, f64) {
    let p93 = if pct_indices.len() >= 4 {
        pct_indices[..4].iter().map(|&i| d_sorted[i]).sum::<f64>() / 4.0
    } else {
        d_sorted.last().copied().unwrap_or(0.0)
    };
    let p83 = if pct_indices.len() >= 8 {
        pct_indices[4..8].iter().map(|&i| d_sorted[i]).sum::<f64>() / 4.0
    } else {
        d_sorted.last().copied().unwrap_or(0.0)
    };
    (p93, p83)
}

fn weighted_mean_power5(d: &[f64], w: &[f64]) -> f64 {
    let (num, den) = d.iter().zip(w).fold((0.0, 0.0), |(n, dw), (&v, &wv)| (n + v.powf(5.0) * wv, dw + wv));
    if den > 0.0 { (num / den).powf(0.2) } else { 0.0 }
}

fn phase4(d_all: &[f64], c_arr: &[f64], all_corners: &[f64]) -> (f64, f64, f64) {
    let gaps = compute_gaps(all_corners);
    let eff_w: Vec<f64> = c_arr.iter().zip(gaps.iter()).map(|(c, g)| c * g).collect();
    let mut indices: Vec<usize> = (0..d_all.len()).collect();
    indices.sort_unstable_by(|&i, &j| d_all[i].partial_cmp(&d_all[j]).expect("finite"));
    let d_sorted: Vec<f64> = indices.iter().map(|&i| d_all[i]).collect();
    let w_sorted: Vec<f64> = indices.iter().map(|&i| eff_w[i]).collect();
    let cum_w = cumulative_weights(&w_sorted);
    let total = cum_w.last().unwrap_or(&1.0);
    let norm_cum: Vec<f64> = cum_w.iter().map(|cw| cw / total).collect();
    let targets = [0.945, 0.935, 0.925, 0.915, 0.845, 0.835, 0.825, 0.815];
    let pct_idx: Vec<usize> = targets.iter().filter_map(|&p| norm_cum.iter().position(|&v| v >= p)).collect();
    let (p93, p83) = percentile_pair(&d_sorted, &pct_idx);
    (p93, p83, weighted_mean_power5(&d_sorted, &w_sorted))
}

#[inline]
fn compute_gaps(all_corners: &[f64]) -> Vec<f64> {
    let n = all_corners.len();
    if n < 2 { return vec![0.0; n]; }
    let mut gaps = Vec::with_capacity(n);
    gaps.push((all_corners[1] - all_corners[0]) / 2.0);
    for i in 1..n - 1 {
        gaps.push((all_corners[i + 1] - all_corners[i - 1]) / 2.0);
    }
    gaps.push((all_corners[n - 1] - all_corners[n - 2]) / 2.0);
    gaps
}

fn phase5(percentile_93: f64, percentile_83: f64, weighted_mean: f64, notes: &[Note], long_notes: &[Note]) -> f64 {
    let mut sr = (0.88 * percentile_93) * 0.25 + (0.94 * percentile_83) * 0.2 + weighted_mean * 0.55;
    sr = sr / 8.0 * 8.0;
    let total_notes: f64 = notes.len() as f64
        + 0.5 * long_notes.iter()
            .map(|note| (note.duration().min(1000).max(0) as f64) / 200.0)
            .sum::<f64>();
    sr *= total_notes / (total_notes + 60.0);
    sr = rescale_high(sr);
    sr *= 0.975;
    sr
}

#[inline]
fn compute_active_columns(key_usage: &std::collections::HashMap<usize, Vec<bool>>, k: usize, n: usize) -> Vec<Vec<usize>> {
    (0..n).map(|i| {
        let mut active = Vec::with_capacity(k);
        for col in 0..k {
            if key_usage.get(&col).map_or(false, |v| v[i]) { active.push(col); }
        }
        active
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_s_positive_values() {
        let s = compute_s(1.0, 1.0, 1.0, 0.5, 4.0, 2.0);
        assert!(s > 0.0, "expected positive s got {s}");
    }

    #[test]
    fn test_compute_s_zero_inputs() {
        let s = compute_s(0.0, 0.0, 1.0, 0.0, 4.0, 2.0);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn test_compute_t_bounded() {
        // t = (a^(3/ks) * x) / (x + s + 1) < a^(3/ks) always
        let t = compute_t(1.0, 0.5, 1.0, 2.0);
        assert!(t >= 0.0);
        assert!(t < 1.0);
    }

    #[test]
    fn test_compute_d_zero_when_both_zero() {
        assert_eq!(compute_d(0.0, 0.0), 0.0);
    }

    #[test]
    fn test_compute_d_increases_with_inputs() {
        let d1 = compute_d(1.0, 0.5);
        let d2 = compute_d(2.0, 0.5);
        assert!(d2 > d1);
    }

    #[test]
    fn test_compute_gaps_basic() {
        let corners = vec![0.0, 1.0, 3.0];
        let gaps = compute_gaps(&corners);
        assert_eq!(gaps.len(), 3);
        assert!((gaps[0] - 0.5).abs() < 1e-9);  // (1.0 - 0.0) / 2
        assert!((gaps[1] - 1.5).abs() < 1e-9);  // (3.0 - 0.0) / 2
        assert!((gaps[2] - 1.0).abs() < 1e-9);  // (3.0 - 1.0) / 2
    }

    #[test]
    fn test_compute_gaps_empty() {
        assert!(compute_gaps(&[]).is_empty());
        assert_eq!(compute_gaps(&[5.0]), vec![0.0]);
    }

    #[test]
    fn test_cumulative_weights_basic() {
        let w = vec![1.0, 2.0, 3.0];
        let cum = cumulative_weights(&w);
        assert_eq!(cum, vec![1.0, 3.0, 6.0]);
    }

    #[test]
    fn test_cumulative_weights_empty() {
        assert!(cumulative_weights(&[]).is_empty());
    }

    #[test]
    fn test_weighted_mean_power5_single_value() {
        // mean of single value d with weight w: (d^5 * w / w)^(1/5) = d
        let d = vec![3.0];
        let w = vec![1.0];
        let result = weighted_mean_power5(&d, &w);
        assert!((result - 3.0).abs() < 1e-9, "expected 3.0 got {result}");
    }

    #[test]
    fn test_weighted_mean_power5_zero_weight() {
        let d = vec![5.0];
        let w = vec![0.0];
        assert_eq!(weighted_mean_power5(&d, &w), 0.0);
    }

    #[test]
    fn test_percentile_pair_not_enough_indices() {
        let d_sorted = vec![1.0, 2.0, 3.0];
        let pct_indices = vec![0, 1];  // < 4
        let (p93, p83) = percentile_pair(&d_sorted, &pct_indices);
        assert_eq!(p93, 3.0);  // fallback to last
        assert_eq!(p83, 3.0);
    }

    #[test]
    fn test_percentile_pair_enough_indices() {
        let d_sorted: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let pct_indices = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let (p93, p83) = percentile_pair(&d_sorted, &pct_indices);
        // p93 = (1+2+3+4)/4 = 2.5
        assert!((p93 - 2.5).abs() < 1e-9);
        // p83 = (5+6+7+8)/4 = 6.5
        assert!((p83 - 6.5).abs() < 1e-9);
    }
}
