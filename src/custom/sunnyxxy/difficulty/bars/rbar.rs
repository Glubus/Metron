use std::cell::RefCell;
use super::super::smoothing::{SmoothMode, smooth_on_corners_into};
use super::super::utils::find_next_note_in_column;
use super::super::note::Note;

thread_local! {
    static RBAR_I_LIST: RefCell<Vec<f64>> = RefCell::new(Vec::new());
    static RBAR_R_STEP: RefCell<Vec<f64>> = RefCell::new(Vec::new());
}

fn fill_i_list(tail_sequence: &[Note], times_by_column: &[Vec<i64>], notes_by_column: &[Vec<Note>], x: f64, i_list: &mut Vec<f64>) {
    i_list.clear();
    for note in tail_sequence.iter() {
        let nxt = find_next_note_in_column(
            (note.column, note.hit_time, note.tail_time),
            &times_by_column[note.column],
            notes_by_column,
        );
        let h_j = nxt.1;
        let i_h = 0.001 * ((note.tail_time - note.hit_time - 80).abs() as f64) / x;
        let i_t = 0.001 * ((h_j - note.tail_time - 80).abs() as f64) / x;
        i_list.push(2.0 / (2.0 + (-5.0 * (i_h - 0.75)).exp() + (-5.0 * (i_t - 0.75)).exp()));
    }
}

fn fill_r_step(tail_sequence: &[Note], base_corners: &[f64], i_list: &[f64], x: f64, r_step: &mut Vec<f64>) {
    let n = base_corners.len();
    r_step.resize(n, 0.0);
    r_step[..n].fill(0.0);
    let x_inv = 1.0 / x;
    let mut left_idx = 0usize;
    let mut right_idx = 0usize;
    for i in 0..tail_sequence.len().saturating_sub(1) {
        let t_start = tail_sequence[i].tail_time as f64;
        let t_end = tail_sequence[i + 1].tail_time as f64;
        while left_idx < n && base_corners[left_idx] < t_start { left_idx += 1; }
        if right_idx < left_idx { right_idx = left_idx; }
        while right_idx < n && base_corners[right_idx] < t_end { right_idx += 1; }
        if left_idx >= right_idx { continue; }
        let delta_r = 0.001 * (t_end - t_start);
        let val = 0.08 / delta_r.sqrt() * x_inv * (1.0 + 0.8 * (i_list[i] + i_list[i + 1]));
        r_step[left_idx..right_idx].fill(val);
    }
}

pub fn compute_rbar(
    _k: usize,
    _t: i64,
    x: f64,
    notes_by_column: &[Vec<Note>],
    times_by_column: &[Vec<i64>],
    tail_sequence: &[Note],
    base_corners: &[f64],
    out: &mut Vec<f64>,
) {
    let n = base_corners.len();
    out.resize(n, 0.0);
    RBAR_I_LIST.with(|il_cell| {
        let mut i_list = il_cell.borrow_mut();
        RBAR_R_STEP.with(|rs_cell| {
            let mut r_step = rs_cell.borrow_mut();
            fill_i_list(tail_sequence, times_by_column, notes_by_column, x, &mut i_list);
            fill_r_step(tail_sequence, base_corners, &i_list, x, &mut r_step);
            smooth_on_corners_into(base_corners, &r_step, 500.0, 0.001, SmoothMode::Sum, out);
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_i_list_empty_tail_sequence() {
        let tail_seq: Vec<Note> = vec![];
        let times: Vec<Vec<i64>> = vec![vec![]];
        let notes: Vec<Vec<Note>> = vec![vec![]];
        let mut result = Vec::new();
        fill_i_list(&tail_seq, &times, &notes, 0.09, &mut result);
        assert!(result.is_empty());
    }

    #[test]
    fn test_compute_r_step_empty() {
        let tail_seq: Vec<Note> = vec![];
        let corners = vec![0.0, 100.0, 200.0];
        let i_list: Vec<f64> = vec![];
        let mut r_step = Vec::new();
        fill_r_step(&tail_seq, &corners, &i_list, 0.09, &mut r_step);
        assert!(r_step.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_compute_r_step_single_note() {
        // single note → saturating_sub(1) = 0 iterations → all zeros
        let tail_seq = vec![Note::long_note(0, 0, 100)];
        let corners = vec![0.0, 100.0, 200.0];
        let i_list = vec![0.5];
        let mut r_step = Vec::new();
        fill_r_step(&tail_seq, &corners, &i_list, 0.09, &mut r_step);
        assert!(r_step.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_compute_r_step_two_notes_fills_range() {
        let tail_seq = vec![
            Note::long_note(0, 0, 50),
            Note::long_note(0, 200, 300),
        ];
        let corners: Vec<f64> = (0..400).map(|i| i as f64).collect();
        let i_list = vec![0.4, 0.4];
        let mut r_step = Vec::new();
        fill_r_step(&tail_seq, &corners, &i_list, 0.09, &mut r_step);
        // Range [50, 300) should be filled
        let filled = r_step[51..299].iter().any(|&v| v > 0.0);
        assert!(filled, "r_step should have nonzero values in range");
    }

    #[test]
    fn test_compute_i_value_bounds() {
        // i value must be in (0, 1)
        // 2 / (2 + exp + exp) — both exp terms positive → result in (0,1)
        let ln_val = 2.0 / (2.0 + (-5.0 * (0.5_f64 - 0.75)).exp() + (-5.0 * (0.5_f64 - 0.75)).exp());
        assert!(ln_val > 0.0 && ln_val < 1.0);
    }
}
