use std::cell::RefCell;

use super::super::note::Note;
use super::super::smoothing::{SmoothMode, smooth_on_corners_into};

thread_local! {
    static XBAR_X_KS: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
    static XBAR_FAST_CROSS: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
    static XBAR_BASE: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
    static XBAR_PAIR_TIMES: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
}

const MAX_COLS: usize = 11;

pub fn compute_xbar(
    key_count: usize,
    hit_leniency: f64,
    notes_by_column: &[Vec<Note>],
    active_masks: &[u16],
    base_corners: &[f64],
) -> Vec<f64> {
    let mut cross_coeff_arr = [0.0f64; MAX_COLS];
    let len = cross_coefficients(key_count, &mut cross_coeff_arr);
    let cross_coeff = &cross_coeff_arr[..len];
    let mut cross_comp_arr = [0.0f64; MAX_COLS];
    for i in 0..len {
        cross_comp_arr[i] = 1.0 - cross_coeff[i];
    }
    let cross_comp = &cross_comp_arr[..len];
    let corner_count = base_corners.len();
    let total_cols = key_count + 1;
    let mut out = vec![0.0; corner_count];

    XBAR_X_KS.with(|xk_cell| {
        let mut x_ks = xk_cell.borrow_mut();
        XBAR_FAST_CROSS.with(|fc_cell| {
            let mut fast_cross = fc_cell.borrow_mut();
            XBAR_BASE.with(|xb_cell| {
                let mut x_base = xb_cell.borrow_mut();
                XBAR_PAIR_TIMES.with(|pt_cell| {
                    let mut pair_times = pt_cell.borrow_mut();
                    let flat_len = total_cols * corner_count;
                    x_ks.resize(flat_len, 0.0);
                    fast_cross.resize(flat_len, 0.0);
                    x_base.resize(corner_count, 0.0);

                    for column in 0..=key_count {
                        let (left, right) = pair_notes(column, key_count, notes_by_column);
                        collect_pair_times_into(left, right, &mut pair_times);
                        fill_column_contributions_into(
                            &pair_times,
                            base_corners,
                            hit_leniency,
                            cross_comp[column],
                            column,
                            active_masks,
                            &mut x_ks[column * corner_count..(column + 1) * corner_count],
                            &mut fast_cross[column * corner_count..(column + 1) * corner_count],
                        );
                    }

                    merge_xbar_contributions_into(
                        &x_ks,
                        &fast_cross,
                        cross_coeff,
                        corner_count,
                        key_count,
                        &mut x_base,
                    );
                    smooth_on_corners_into(
                        base_corners,
                        &x_base,
                        500.0,
                        0.001,
                        SmoothMode::Sum,
                        &mut out,
                    );
                });
            });
        });
    });

    out
}

fn cross_coefficients(key_count: usize, out: &mut [f64; MAX_COLS]) -> usize {
    let len = key_count + 1;
    match key_count {
        0 => out[..1].copy_from_slice(&[-1.0]),
        1 => out[..2].copy_from_slice(&[0.075, 0.075]),
        2 => out[..3].copy_from_slice(&[0.125, 0.05, 0.125]),
        3 => out[..4].copy_from_slice(&[0.125, 0.125, 0.125, 0.125]),
        4 => out[..5].copy_from_slice(&[0.175, 0.25, 0.05, 0.25, 0.175]),
        5 => out[..6].copy_from_slice(&[0.175, 0.25, 0.175, 0.175, 0.25, 0.175]),
        6 => out[..7].copy_from_slice(&[0.225, 0.35, 0.25, 0.05, 0.25, 0.35, 0.225]),
        7 => out[..8].copy_from_slice(&[0.225, 0.35, 0.25, 0.225, 0.225, 0.25, 0.35, 0.225]),
        8 => out[..9].copy_from_slice(&[0.275, 0.45, 0.35, 0.25, 0.05, 0.25, 0.35, 0.45, 0.275]),
        9 => out[..10].copy_from_slice(&[
            0.275, 0.45, 0.35, 0.25, 0.275, 0.275, 0.25, 0.35, 0.45, 0.275,
        ]),
        10 => out[..11].copy_from_slice(&[
            0.325, 0.55, 0.45, 0.35, 0.25, 0.05, 0.25, 0.35, 0.45, 0.55, 0.325,
        ]),
        _ => panic!("unsupported key count {key_count}"),
    }
    len
}

fn pair_notes<'a>(
    column: usize,
    key_count: usize,
    notes_by_column: &'a [Vec<Note>],
) -> (&'a [Note], &'a [Note]) {
    if column == 0 {
        (&notes_by_column[0], &[])
    } else if column == key_count {
        (&notes_by_column[key_count - 1], &[])
    } else {
        (&notes_by_column[column - 1], &notes_by_column[column])
    }
}

fn collect_pair_times_into(left: &[Note], right: &[Note], times: &mut Vec<f64>) {
    times.clear();
    let (mut left_index, mut right_index) = (0usize, 0usize);
    while left_index < left.len() || right_index < right.len() {
        if left_index < left.len()
            && (right_index >= right.len()
                || left[left_index].hit_time <= right[right_index].hit_time)
        {
            times.push(left[left_index].hit_time as f64);
            left_index += 1;
        } else {
            times.push(right[right_index].hit_time as f64);
            right_index += 1;
        }
    }
}

fn fill_column_contributions_into(
    times: &[f64],
    base_corners: &[f64],
    hit_leniency: f64,
    cross_comp: f64,
    column: usize,
    active_masks: &[u16],
    x_col: &mut [f64],
    fast_col: &mut [f64],
) {
    x_col.fill(0.0);
    fast_col.fill(0.0);
    if times.len() < 2 {
        return;
    }

    let mut start_index = 0usize;
    let mut end_index = 0usize;
    for pair in times.windows(2) {
        let (prev_time, next_time) = (pair[0], pair[1]);
        while start_index < base_corners.len() && base_corners[start_index] < prev_time {
            start_index += 1;
        }
        if end_index < start_index {
            end_index = start_index;
        }
        while end_index < base_corners.len() && base_corners[end_index] < next_time {
            end_index += 1;
        }
        if start_index >= end_index {
            continue;
        }

        fill_interval(
            x_col,
            fast_col,
            start_index,
            end_index,
            prev_time,
            next_time,
            hit_leniency,
            cross_comp,
            column,
            active_masks,
        );
    }
}

fn fill_interval(
    x_col: &mut [f64],
    fast_col: &mut [f64],
    start_index: usize,
    end_index: usize,
    prev_time: f64,
    next_time: f64,
    hit_leniency: f64,
    cross_comp: f64,
    column: usize,
    active_masks: &[u16],
) {
    let delta = 0.001 * (next_time - prev_time);
    let mut value = 0.16 / hit_leniency.max(delta).powi(2);
    if is_inactive_neighbor_pair(active_masks, start_index, end_index, column) {
        value *= cross_comp;
    }
    let fast_value = (0.4 / delta.max(0.06).max(0.75 * hit_leniency).powi(2) - 80.0).max(0.0);
    x_col[start_index..end_index].fill(value);
    fast_col[start_index..end_index].fill(fast_value);
}

fn is_inactive_neighbor_pair(
    active_masks: &[u16],
    start_index: usize,
    end_index: usize,
    column: usize,
) -> bool {
    let previous_bit = if column > 0 {
        1u16 << (column - 1)
    } else {
        0u16
    };
    let current_bit = 1u16 << column;
    let start_mask = active_masks.get(start_index).copied().unwrap_or(0);
    let end_mask = active_masks.get(end_index).copied().unwrap_or(0);
    let previous_inactive = (start_mask & previous_bit == 0) && (end_mask & previous_bit == 0);
    let current_inactive = (start_mask & current_bit == 0) && (end_mask & current_bit == 0);
    previous_inactive || current_inactive
}

fn merge_xbar_contributions_into(
    x_ks: &[f64],
    fast_cross: &[f64],
    cross_coeff: &[f64],
    corner_count: usize,
    key_count: usize,
    out: &mut [f64],
) {
    out[..corner_count].fill(0.0);
    for column in 0..=key_count {
        let coeff = cross_coeff[column];
        let x_col = &x_ks[column * corner_count..(column + 1) * corner_count];
        for i in 0..corner_count {
            out[i] += x_col[i] * coeff;
        }
    }
    for column in 0..key_count {
        let left_coeff = cross_coeff[column];
        let right_coeff = cross_coeff[column + 1];
        let left = &fast_cross[column * corner_count..(column + 1) * corner_count];
        let right = &fast_cross[(column + 1) * corner_count..(column + 2) * corner_count];
        for i in 0..corner_count {
            out[i] += (left[i] * left_coeff * right[i] * right_coeff).sqrt();
        }
    }
}
