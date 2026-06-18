use super::super::interpolation::search_left;
use super::super::smoothing::{SmoothMode, smooth_on_corners_into};

pub fn compute_abar(
    key_count: usize,
    active_masks: &[u16],
    delta_ks: &[Vec<f64>],
    a_corners: &[f64],
    base_corners: &[f64],
) -> Vec<f64> {
    let mut dks = vec![vec![0.0; base_corners.len()]; key_count.saturating_sub(1)];
    build_dks(
        key_count,
        active_masks,
        delta_ks,
        base_corners.len(),
        &mut dks,
    );

    let base_indices: Vec<usize> = a_corners
        .iter()
        .map(|&corner| search_left(base_corners, corner).min(base_corners.len() - 1))
        .collect();
    let mut a_step = vec![1.0; a_corners.len()];

    for (i, &base_index) in base_indices.iter().enumerate() {
        a_step[i] = abar_factor_for_corner(active_masks[base_index], base_index, &dks, delta_ks);
    }

    let mut out = vec![0.0; a_corners.len()];
    smooth_on_corners_into(a_corners, &a_step, 250.0, 1.0, SmoothMode::Avg, &mut out);
    out
}

fn build_dks(
    key_count: usize,
    active_masks: &[u16],
    delta_ks: &[Vec<f64>],
    corner_count: usize,
    dks: &mut [Vec<f64>],
) {
    for dk in dks.iter_mut() {
        dk.resize(corner_count, 0.0);
        dk[..corner_count].fill(0.0);
    }

    for (index, &mask) in active_masks.iter().enumerate() {
        for_each_adjacent_pair(mask, key_count, |left, right| {
            dks[left][index] = pair_delta_difference(delta_ks, left, right, index);
        });
    }
}

fn abar_factor_for_corner(
    active_mask: u16,
    base_index: usize,
    dks: &[Vec<f64>],
    delta_ks: &[Vec<f64>],
) -> f64 {
    let mut factor = 1.0;
    for_each_adjacent_pair(active_mask, delta_ks.len(), |left, right| {
        factor *= pair_anchor_factor(
            dks[left][base_index],
            delta_ks[left][base_index],
            delta_ks[right][base_index],
        );
    });
    factor
}

fn pair_delta_difference(delta_ks: &[Vec<f64>], left: usize, right: usize, index: usize) -> f64 {
    (delta_ks[left][index] - delta_ks[right][index]).abs()
        + 0.4 * (delta_ks[left][index].max(delta_ks[right][index]) - 0.11).max(0.0)
}

fn pair_anchor_factor(d_val: f64, left_delta: f64, right_delta: f64) -> f64 {
    if d_val < 0.02 {
        (0.75 + 0.5 * left_delta.max(right_delta)).min(1.0)
    } else if d_val < 0.07 {
        (0.65 + 5.0 * d_val + 0.5 * left_delta.max(right_delta)).min(1.0)
    } else {
        1.0
    }
}

fn for_each_adjacent_pair(mask: u16, key_count: usize, mut callback: impl FnMut(usize, usize)) {
    let mut previous = None;
    for column in 0..key_count {
        if mask & (1u16 << column) == 0 {
            continue;
        }
        if let Some(left) = previous {
            callback(left, column);
        }
        previous = Some(column);
    }
}
