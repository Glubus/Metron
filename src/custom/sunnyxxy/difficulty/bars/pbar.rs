use super::super::calculations::ln::ln_sum;
use super::super::smoothing::{SmoothMode, smooth_on_corners};
use super::super::note::Note;

fn stream_booster(delta: f64) -> f64 {
    let r = 7.5 / delta;
    if r > 160.0 && r < 360.0 {
        1.0 + 1.7e-7 * (r - 160.0) * (r - 360.0).powi(2)
    } else {
        1.0
    }
}

fn simultaneous_spike(x: f64) -> f64 {
    1000.0 * (0.02 * (4.0 / x - 24.0)).powf(0.25)
}

fn pbar_increment(delta: f64, v: f64, b_val: f64, x: f64, x_inv: f64, x_inv_quarter: f64) -> f64 {
    let two_thirds_x = (2.0 / 3.0) * x;
    let x_over_six = x / 6.0;
    let inv_delta = 1.0 / delta.max(1e-12);
    let inner = if delta < two_thirds_x {
        1.0 - 24.0 * x_inv * (delta - x * 0.5).powi(2)
    } else {
        1.0 - 24.0 * x_inv * (x_over_six * x_over_six)
    };
    inv_delta * (x_inv_quarter * inner.max(0.0).powf(0.25)) * b_val.max(v)
}

fn apply_pair_step(p_step: &mut [f64], anchor: &[f64], left: usize, right: usize, inc: f64) {
    for idx in left..right {
        let add = inc * anchor[idx];
        p_step[idx] += add.min((inc * 2.0 - 10.0).max(inc));
    }
}

fn advance_window(corners: &[f64], left: &mut usize, right: &mut usize, start: f64, end: f64) -> bool {
    while *left < corners.len() && corners[*left] < start { *left += 1; }
    if *right < *left { *right = *left; }
    while *right < corners.len() && corners[*right] < end { *right += 1; }
    *left < *right
}

fn handle_simultaneous(p_step: &mut [f64], corners: &[f64], left: &mut usize, right: &mut usize, h_l: f64, x: f64) {
    let spike = simultaneous_spike(x);
    while *left < corners.len() && corners[*left] < h_l { *left += 1; }
    if *right < *left { *right = *left; }
    while *right < corners.len() && corners[*right] <= h_l { *right += 1; }
    for idx in *left..*right { p_step[idx] += spike; }
}

fn process_note_pair(p_step: &mut [f64], anchor: &[f64], corners: &[f64], left: &mut usize, right: &mut usize, h_l: f64, h_r: f64, v: f64, x: f64, x_inv: f64, x_inv_quarter: f64) {
    if !advance_window(corners, left, right, h_l, h_r) { return; }
    let delta = 0.001 * (h_r - h_l);
    let inc = pbar_increment(delta, v, stream_booster(delta), x, x_inv, x_inv_quarter);
    apply_pair_step(p_step, anchor, *left, *right, inc);
}

pub fn compute_pbar(
    _k: usize,
    _t: i64,
    x: f64,
    notes: &[Note],
    ln_rep: &(Vec<i64>, Vec<f64>, Vec<f64>),
    anchor: &[f64],
    base_corners: &[f64],
) -> Vec<f64> {
    let mut p_step = vec![0.0; base_corners.len()];
    let x_inv = 1.0 / x.max(1e-12);
    let x_inv_quarter = (0.08 * x_inv).powf(0.25);
    let mut left_idx = 0usize;
    let mut right_idx = 0usize;
    for i in 0..notes.len().saturating_sub(1) {
        let (h_l, h_r) = (notes[i].hit_time as f64, notes[i + 1].hit_time as f64);
        if (h_r - h_l).abs() < 1e-9 {
            handle_simultaneous(&mut p_step, base_corners, &mut left_idx, &mut right_idx, h_l, x);
        } else {
            let v = 1.0 + 6.0 * 0.001 * ln_sum(h_l, h_r, ln_rep);
            process_note_pair(&mut p_step, anchor, base_corners, &mut left_idx, &mut right_idx, h_l, h_r, v, x, x_inv, x_inv_quarter);
        }
    }
    smooth_on_corners(base_corners, &p_step, 500.0, 0.001, SmoothMode::Sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_booster_outside_range() {
        assert_eq!(stream_booster(1.0), 1.0);
        assert_eq!(stream_booster(0.1), 1.0);
    }

    #[test]
    fn test_stream_booster_at_boundary_low() {
        // r = exactly 160.0: not strictly greater → 1.0
        assert_eq!(stream_booster(7.5 / 160.0), 1.0);
    }

    #[test]
    fn test_stream_booster_at_boundary_high() {
        // r = exactly 360.0: not strictly less → 1.0
        assert_eq!(stream_booster(7.5 / 360.0), 1.0);
    }

    #[test]
    fn test_stream_booster_inside_range() {
        // delta = 0.0375, r = 200.0
        // 1.0 + 1.7e-7 * (200-160) * (200-360)^2 = 1.0 + 1.7e-7 * 40 * 25600 ≈ 1.174
        let boost = stream_booster(0.0375);
        assert!(boost > 1.0, "expected boost > 1.0, got {boost}");
        assert!((boost - 1.174).abs() < 0.01, "expected ~1.174, got {boost}");
    }

    #[test]
    fn test_simultaneous_spike_positive() {
        assert!(simultaneous_spike(0.1) > 0.0);
        assert!(simultaneous_spike(0.09) > 0.0);
    }

    #[test]
    fn test_pbar_increment_positive() {
        let x = 0.09_f64;
        let x_inv = 1.0 / x;
        let x_inv_quarter = (0.08 * x_inv).powf(0.25);
        let inc = pbar_increment(0.5, 1.0, 1.0, x, x_inv, x_inv_quarter);
        assert!(inc > 0.0);
    }

    #[test]
    fn test_pbar_increment_higher_boost_gives_higher_inc() {
        let x = 0.09_f64;
        let x_inv = 1.0 / x;
        let x_inv_quarter = (0.08 * x_inv).powf(0.25);
        let inc_low = pbar_increment(0.5, 1.0, 1.0, x, x_inv, x_inv_quarter);
        let inc_high = pbar_increment(0.5, 2.0, 2.0, x, x_inv, x_inv_quarter);
        assert!(inc_high > inc_low);
    }

    #[test]
    fn test_advance_window_returns_false_when_empty_range() {
        let corners = vec![0.0, 10.0, 20.0];
        let mut l = 0;
        let mut r = 0;
        // start and end both after all corners
        assert!(!advance_window(&corners, &mut l, &mut r, 25.0, 30.0));
    }

    #[test]
    fn test_advance_window_returns_true_when_range_found() {
        let corners = vec![0.0, 10.0, 20.0];
        let mut l = 0;
        let mut r = 0;
        assert!(advance_window(&corners, &mut l, &mut r, 0.0, 15.0));
        assert!(l < r);
    }

    #[test]
    fn test_apply_pair_step_no_anchor() {
        let mut p_step = vec![0.0; 3];
        let anchor = vec![1.0; 3];
        apply_pair_step(&mut p_step, &anchor, 0, 3, 5.0);
        // add = 5.0 * 1.0 = 5.0; alt = (5.0 * 2.0 - 10.0).max(5.0) = 0.0.max(5.0) = 5.0
        // min(5.0, 5.0) = 5.0
        assert_eq!(p_step[0], 5.0);
    }
}
