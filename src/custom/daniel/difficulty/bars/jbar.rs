use std::cell::RefCell;

use super::super::note::Note;
use super::super::smoothing::{SmoothMode, smooth_on_corners_into};

const LARGE_DELTA: f64 = 1e9;

thread_local! {
    static JBAR_SMOOTH: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
    static JBAR_DEN: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
    static JBAR_J_COL: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
    static JBAR_DELTA_COL: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
}

pub fn compute_jbar(
    key_count: usize,
    hit_leniency: f64,
    notes_by_column: &[Vec<Note>],
    base_corners: &[f64],
) -> (Vec<Vec<f64>>, Vec<f64>) {
    let corner_count = base_corners.len();
    let mut delta_ks = vec![vec![LARGE_DELTA; corner_count]; key_count];
    let mut jbar = vec![0.0; corner_count];
    let x_quarter = hit_leniency.sqrt().sqrt();

    JBAR_J_COL.with(|jc_cell| {
        let mut j_col = jc_cell.borrow_mut();
        JBAR_DELTA_COL.with(|dc_cell| {
            let mut delta_col = dc_cell.borrow_mut();
            JBAR_SMOOTH.with(|s_cell| {
                let mut smooth_tmp = s_cell.borrow_mut();
                smooth_tmp.resize(corner_count, 0.0);
                JBAR_DEN.with(|d_cell| {
                    let mut den = d_cell.borrow_mut();
                    den.resize(corner_count, 0.0);
                    den[..corner_count].fill(0.0);
                    jbar[..corner_count].fill(0.0);

                    for column in 0..key_count {
                        compute_column_jack_into(
                            &notes_by_column[column],
                            base_corners,
                            x_quarter,
                            &mut j_col,
                            &mut delta_col,
                        );
                        smooth_on_corners_into(
                            base_corners,
                            &j_col,
                            500.0,
                            0.001,
                            SmoothMode::Sum,
                            &mut smooth_tmp,
                        );
                        accumulate_jbar_column(&smooth_tmp, &delta_col, &mut jbar, &mut den);
                        delta_ks[column].resize(corner_count, LARGE_DELTA);
                        delta_ks[column].copy_from_slice(&delta_col[..corner_count]);
                    }

                    finalize_jbar(&mut jbar, &den);
                });
            });
        });
    });

    (delta_ks, jbar)
}

fn jack_nerfer(delta: f64) -> f64 {
    let d = 0.15 + (delta - 0.08).abs();
    let d2 = d * d;
    1.0 - 7e-5 / (d2 * d2)
}

fn compute_column_jack_into(
    notes: &[Note],
    base_corners: &[f64],
    x_quarter: f64,
    j_col: &mut Vec<f64>,
    delta_col: &mut Vec<f64>,
) {
    let corner_count = base_corners.len();
    j_col.resize(corner_count, 0.0);
    j_col[..corner_count].fill(0.0);
    delta_col.resize(corner_count, LARGE_DELTA);
    delta_col[..corner_count].fill(LARGE_DELTA);
    if notes.len() < 2 {
        return;
    }

    let mut left_index = 0usize;
    let mut right_index = 0usize;
    for pair in notes.windows(2) {
        let start = pair[0].hit_time as f64;
        let end = pair[1].hit_time as f64;
        while left_index < corner_count && base_corners[left_index] < start {
            left_index += 1;
        }
        if right_index < left_index {
            right_index = left_index;
        }
        while right_index < corner_count && base_corners[right_index] < end {
            right_index += 1;
        }
        if left_index >= right_index {
            continue;
        }

        let delta = 0.001 * (end - start);
        let inv_delta = 1.0 / delta.max(1e-12);
        let j_value = inv_delta / (delta + 0.11 * x_quarter).max(1e-12) * jack_nerfer(delta);
        j_col[left_index..right_index].fill(j_value);
        delta_col[left_index..right_index].fill(delta);
    }
}

fn accumulate_jbar_column(smooth: &[f64], delta_col: &[f64], jbar: &mut [f64], den: &mut [f64]) {
    for i in 0..jbar.len() {
        let weight = 1.0 / delta_col[i];
        jbar[i] += smooth[i].max(0.0).powi(5) * weight;
        den[i] += weight;
    }
}

fn finalize_jbar(jbar: &mut [f64], den: &[f64]) {
    for i in 0..jbar.len() {
        jbar[i] = (jbar[i] / den[i].max(1e-9)).powf(0.2);
    }
}
