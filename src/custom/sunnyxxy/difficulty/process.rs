use std::cell::RefCell;
use super::interpolation::{interp_values_into, step_interp_into};
use super::smoothing::rescale_high;
use super::map_data::MapData;
use super::note::Note;
use super::bars::abar::compute_abar;
use super::bars::jbar::compute_jbar;
use super::bars::pbar::compute_pbar;
use super::bars::rbar::compute_rbar;
use super::bars::xbar::compute_xbar;
use super::calculations::anchor::compute_anchor_into;
use super::calculations::ck::compute_c_and_ks;
use super::calculations::corners::get_corners_into;
use super::calculations::key_usage::{with_key_usage, with_key_usage_400};
use super::calculations::ln::ln_bodies_count_sparse_representation;

struct BarOutputs {
    jbar: Vec<f64>,
    xbar: Vec<f64>,
    pbar: Vec<f64>,
    abar: Vec<f64>,
    rbar: Vec<f64>,
    c_arr: Vec<f64>,
    ks_arr: Vec<f64>,
}

impl BarOutputs {
    fn new() -> Self {
        Self {
            jbar: Vec::new(), xbar: Vec::new(), pbar: Vec::new(),
            abar: Vec::new(), rbar: Vec::new(), c_arr: Vec::new(), ks_arr: Vec::new(),
        }
    }
}

struct ScratchBuffers {
    d_all: Vec<f64>,
    eff_w: Vec<f64>,
    indices: Vec<usize>,
    cum_w: Vec<f64>,
}

impl ScratchBuffers {
    fn new() -> Self {
        Self { d_all: Vec::new(), eff_w: Vec::new(), indices: Vec::new(), cum_w: Vec::new() }
    }
}

struct BaseBarScratch {
    jbar_base: Vec<f64>,
    xbar_base: Vec<f64>,
    pbar_base: Vec<f64>,
    abar_base: Vec<f64>,
    rbar_base: Vec<f64>,
    c_step: Vec<f64>,
    ks_step: Vec<f64>,
}

impl BaseBarScratch {
    fn new() -> Self {
        Self {
            jbar_base: Vec::new(), xbar_base: Vec::new(), pbar_base: Vec::new(),
            abar_base: Vec::new(), rbar_base: Vec::new(), c_step: Vec::new(), ks_step: Vec::new(),
        }
    }
}

struct Phase1Scratch {
    all_corners: Vec<f64>,
    base_corners: Vec<f64>,
    a_corners: Vec<f64>,
    active_columns: Vec<u16>,
    anchor: Vec<f64>,
}

impl Phase1Scratch {
    fn new() -> Self {
        Self {
            all_corners: Vec::new(), base_corners: Vec::new(), a_corners: Vec::new(),
            active_columns: Vec::new(), anchor: Vec::new(),
        }
    }
}

thread_local! {
    static BAR_OUTPUTS: RefCell<BarOutputs> = RefCell::new(BarOutputs::new());
    static SCRATCH: RefCell<ScratchBuffers> = RefCell::new(ScratchBuffers::new());
    static DELTA_KS_BUF: RefCell<Vec<Vec<f64>>> = RefCell::new(Vec::new());
    static BASE_BAR_SCRATCH: RefCell<BaseBarScratch> = RefCell::new(BaseBarScratch::new());
    static PHASE1_SCRATCH: RefCell<Phase1Scratch> = RefCell::new(Phase1Scratch::new());
}

pub fn calculate_internal(map_data: &MapData) -> f64 {
    PHASE1_SCRATCH.with(|p1_cell| {
        let mut p1 = p1_cell.borrow_mut();
        phase1(map_data, &mut p1);
        DELTA_KS_BUF.with(|dk_cell| {
            let mut delta_ks = dk_cell.borrow_mut();
            BASE_BAR_SCRATCH.with(|bbs_cell| {
                let mut base = bbs_cell.borrow_mut();
                BAR_OUTPUTS.with(|bo_cell| {
                    let mut bars = bo_cell.borrow_mut();
                    fill_phase2(map_data, &p1.active_columns, &p1.a_corners, &p1.base_corners, &p1.all_corners, &p1.anchor, &mut bars, &mut base, &mut delta_ks);
                    SCRATCH.with(|sc_cell| {
                        let mut sc = sc_cell.borrow_mut();
                        phase3_into(&bars.jbar, &bars.xbar, &bars.pbar, &bars.abar, &bars.rbar, &bars.c_arr, &bars.ks_arr, &mut sc.d_all);
                        let ScratchBuffers { ref d_all, ref mut eff_w, ref mut indices, ref mut cum_w } = *sc;
                        let (p93, p83, wm) = phase4_into(d_all, &bars.c_arr, &p1.all_corners, eff_w, indices, cum_w);
                        phase5(p93, p83, wm, &map_data.notes, &map_data.long_notes)
                    })
                })
            })
        })
    })
}

fn phase1(map_data: &MapData, p1: &mut Phase1Scratch) {
    get_corners_into(map_data.total_duration, &map_data.notes, &mut p1.all_corners, &mut p1.base_corners, &mut p1.a_corners);
    let k = map_data.column_count;
    let n = p1.base_corners.len();
    with_key_usage(k, map_data.total_duration, &map_data.notes, &p1.base_corners, |ku| {
        compute_active_columns_into(ku, k, n, &mut p1.active_columns);
    });
    with_key_usage_400(k, map_data.total_duration, &map_data.notes, &p1.base_corners, |ku400| {
        compute_anchor_into(k, ku400, n, &p1.base_corners, &mut p1.anchor);
    });
}

fn fill_phase2(map_data: &MapData, active_columns: &[u16], a_corners: &[f64], base_corners: &[f64], all_corners: &[f64], anchor: &[f64], bars: &mut BarOutputs, base: &mut BaseBarScratch, delta_ks: &mut Vec<Vec<f64>>) {
    compute_jbar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes_by_column, base_corners, delta_ks, &mut base.jbar_base);
    interp_values_into(all_corners, base_corners, &base.jbar_base, &mut bars.jbar);

    compute_xbar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes_by_column, active_columns, base_corners, &mut base.xbar_base);
    interp_values_into(all_corners, base_corners, &base.xbar_base, &mut bars.xbar);

    let ln_rep = ln_bodies_count_sparse_representation(&map_data.long_notes, map_data.total_duration);
    compute_pbar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes, &ln_rep, anchor, base_corners, &mut base.pbar_base);
    interp_values_into(all_corners, base_corners, &base.pbar_base, &mut bars.pbar);

    compute_abar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes_by_column, active_columns, &delta_ks, a_corners, base_corners, &mut base.abar_base);
    interp_values_into(all_corners, a_corners, &base.abar_base, &mut bars.abar);

    compute_rbar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes_by_column, &map_data.times_by_column, &map_data.tail_sequence, base_corners, &mut base.rbar_base);
    interp_values_into(all_corners, base_corners, &base.rbar_base, &mut bars.rbar);

    compute_c_and_ks(map_data.column_count, map_data.total_duration, &map_data.notes, active_columns, base_corners, &mut base.c_step, &mut base.ks_step);
    step_interp_into(all_corners, base_corners, &base.c_step, &mut bars.c_arr);
    step_interp_into(all_corners, base_corners, &base.ks_step, &mut bars.ks_arr);
}

fn compute_s(j: f64, p: f64, a: f64, r: f64, c: f64, ks: f64, a_pow_3_ks: f64) -> f64 {
    let jack_term = a_pow_3_ks * j.min(8.0 + 0.85 * j);
    let stream_term = (a * a).cbrt() * (0.8 * p + r * 35.0 / (c + 8.0));
    let jt15 = jack_term * jack_term.sqrt();
    let st15 = stream_term * stream_term.sqrt();
    let sum = 0.4 * jt15 + 0.6 * st15;
    (sum * sum).cbrt()
}

#[inline]
fn compute_t(s: f64, x: f64, a_pow_3_ks: f64) -> f64 {
    (a_pow_3_ks * x) / (x + s + 1.0)
}

fn compute_d(s: f64, t: f64) -> f64 {
    2.7 * s.sqrt() * (t * t.sqrt()) + s * 0.27
}

#[inline]
fn pow_3_div_ks(a: f64, ks: f64) -> f64 {
    match ks as u32 {
        1 => a * a * a,
        2 => a * a.sqrt(),
        3 => a,
        4 => { let s = a.sqrt(); s * s.sqrt() },
        _ => a.powf(3.0 / ks),
    }
}

fn phase3_into(jbar: &[f64], xbar: &[f64], pbar: &[f64], abar: &[f64], rbar: &[f64], c_arr: &[f64], ks_arr: &[f64], out: &mut Vec<f64>) {
    let n = jbar.len();
    out.resize(n, 0.0);
    let iter = jbar.iter().zip(xbar).zip(pbar).zip(abar).zip(rbar).zip(c_arr).zip(ks_arr);
    for (i, ((((((&j, &x), &p), &a), &r), &c), &ks)) in iter.enumerate() {
        let a_pow_3_ks = pow_3_div_ks(a, ks);
        let s = compute_s(j, p, a, r, c, ks, a_pow_3_ks);
        let t = compute_t(s, x, a_pow_3_ks);
        out[i] = compute_d(s, t);
    }
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
    let (num, den) = d.iter().zip(w).fold((0.0, 0.0), |(n, dw), (&v, &wv)| (n + v*v*v*v*v * wv, dw + wv));
    if den > 0.0 { (num / den).powf(0.2) } else { 0.0 }
}

fn phase4_into(d_all: &[f64], c_arr: &[f64], all_corners: &[f64], eff_w: &mut Vec<f64>, indices: &mut Vec<usize>, cum_w: &mut Vec<f64>) -> (f64, f64, f64) {
    let n = all_corners.len();
    eff_w.clear();
    if n < 2 {
        eff_w.resize(n, 0.0);
    } else {
        eff_w.push(c_arr[0] * (all_corners[1] - all_corners[0]) / 2.0);
        for i in 1..n - 1 {
            eff_w.push(c_arr[i] * (all_corners[i + 1] - all_corners[i - 1]) / 2.0);
        }
        eff_w.push(c_arr[n - 1] * (all_corners[n - 1] - all_corners[n - 2]) / 2.0);
    }
    indices.clear();
    indices.extend(0..n);
    indices.sort_unstable_by(|&i, &j| d_all[i].partial_cmp(&d_all[j]).expect("finite"));
    let mut cum = 0.0;
    cum_w.clear();
    for &i in indices.iter() { cum += eff_w[i]; cum_w.push(cum); }
    let total = cum_w.last().copied().unwrap_or(1.0);
    let targets = [0.945, 0.935, 0.925, 0.915, 0.845, 0.835, 0.825, 0.815];
    let mut pct_idx = [usize::MAX; 8];
    let mut pct_count = 0usize;
    for &p in &targets {
        let idx = cum_w.partition_point(|&v| v < p * total);
        if idx < n { pct_idx[pct_count] = idx; pct_count += 1; }
    }
    let last_d = d_all[*indices.last().unwrap_or(&0)];
    let p93 = if pct_count >= 4 {
        pct_idx[..4].iter().map(|&i| d_all[indices[i]]).sum::<f64>() / 4.0
    } else { last_d };
    let p83 = if pct_count >= 8 {
        pct_idx[4..8].iter().map(|&i| d_all[indices[i]]).sum::<f64>() / 4.0
    } else { last_d };
    let (num, den) = indices.iter().fold((0.0, 0.0), |(n, dw), &idx| {
        let v = d_all[idx]; (n + v*v*v*v*v * eff_w[idx], dw + eff_w[idx])
    });
    let wm = if den > 0.0 { (num / den).powf(0.2) } else { 0.0 };
    (p93, p83, wm)
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
    let total_notes: f64 = notes.len() as f64
        + 0.5 * long_notes.iter()
            .map(|note| (note.duration().min(1000).max(0) as f64) / 200.0)
            .sum::<f64>();
    sr *= total_notes / (total_notes + 60.0);
    sr = rescale_high(sr);
    sr *= 0.975;
    sr
}

/// `key_usage` is flat col-major: index = col * n + i
#[inline]
fn compute_active_columns_into(key_usage: &[bool], k: usize, n: usize, out: &mut Vec<u16>) {
    out.resize(n, 0);
    out[..n].fill(0);
    for col in 0..k {
        let col_bit = 1u16 << col;
        let base = col * n;
        for i in 0..n {
            if key_usage[base + i] { out[i] |= col_bit; }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_s_positive_values() {
        let a_pow = 1.0f64.powf(3.0 / 2.0);
        let s = compute_s(1.0, 1.0, 1.0, 0.5, 4.0, 2.0, a_pow);
        assert!(s > 0.0, "expected positive s got {s}");
    }

    #[test]
    fn test_compute_s_zero_inputs() {
        let a_pow = 1.0f64.powf(3.0 / 2.0);
        let s = compute_s(0.0, 0.0, 1.0, 0.0, 4.0, 2.0, a_pow);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn test_compute_t_bounded() {
        // t = (a^(3/ks) * x) / (x + s + 1) < a^(3/ks) always
        let a_pow = 1.0f64.powf(3.0 / 2.0);
        let t = compute_t(1.0, 0.5, a_pow);
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
