use super::super::smoothing::{SmoothMode, smooth_on_corners};
use super::super::utils::find_next_note_in_column;
use super::super::note::Note;

fn compute_i_list(tail_sequence: &[Note], times_by_column: &[Vec<i64>], notes_by_column: &[Vec<Note>], x: f64) -> Vec<f64> {
    tail_sequence.iter().map(|note| {
        let nxt = find_next_note_in_column(
            (note.column, note.hit_time, note.tail_time),
            &times_by_column[note.column],
            notes_by_column,
        );
        let h_j = nxt.1;
        let i_h = 0.001 * ((note.tail_time - note.hit_time - 80).abs() as f64) / x;
        let i_t = 0.001 * ((h_j - note.tail_time - 80).abs() as f64) / x;
        2.0 / (2.0 + (-5.0 * (i_h - 0.75)).exp() + (-5.0 * (i_t - 0.75)).exp())
    }).collect()
}

fn compute_r_step(tail_sequence: &[Note], base_corners: &[f64], i_list: &[f64], x: f64) -> Vec<f64> {
    let mut r_step = vec![0.0; base_corners.len()];
    for i in 0..tail_sequence.len().saturating_sub(1) {
        let t_start = tail_sequence[i].tail_time;
        let t_end = tail_sequence[i + 1].tail_time;
        let left_idx = base_corners.partition_point(|&v| v < t_start as f64);
        let right_idx = base_corners.partition_point(|&v| v < t_end as f64);
        if left_idx >= right_idx { continue; }
        let delta_r = 0.001 * ((tail_sequence[i + 1].tail_time - tail_sequence[i].tail_time) as f64);
        let val = 0.08 * delta_r.powf(-0.5) * x.powf(-1.0) * (1.0 + 0.8 * (i_list[i] + i_list[i + 1]));
        for idx in left_idx..right_idx { r_step[idx] = val; }
    }
    r_step
}

pub fn compute_rbar(
    _k: usize,
    _t: i64,
    x: f64,
    notes_by_column: &[Vec<Note>],
    tail_sequence: &[Note],
    base_corners: &[f64],
) -> Vec<f64> {
    let times_by_column: Vec<Vec<i64>> = notes_by_column.iter()
        .map(|col| col.iter().map(|note| note.hit_time).collect())
        .collect();
    let i_list = compute_i_list(tail_sequence, &times_by_column, notes_by_column, x);
    let r_step = compute_r_step(tail_sequence, base_corners, &i_list, x);
    smooth_on_corners(base_corners, &r_step, 500.0, 0.001, SmoothMode::Sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_i_list_empty_tail_sequence() {
        let tail_seq: Vec<Note> = vec![];
        let times: Vec<Vec<i64>> = vec![vec![]];
        let notes: Vec<Vec<Note>> = vec![vec![]];
        let result = compute_i_list(&tail_seq, &times, &notes, 0.09);
        assert!(result.is_empty());
    }

    #[test]
    fn test_compute_r_step_empty() {
        let tail_seq: Vec<Note> = vec![];
        let corners = vec![0.0, 100.0, 200.0];
        let i_list: Vec<f64> = vec![];
        let r_step = compute_r_step(&tail_seq, &corners, &i_list, 0.09);
        assert!(r_step.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_compute_r_step_single_note() {
        // single note → saturating_sub(1) = 0 iterations → all zeros
        let tail_seq = vec![Note::long_note(0, 0, 100)];
        let corners = vec![0.0, 100.0, 200.0];
        let i_list = vec![0.5];
        let r_step = compute_r_step(&tail_seq, &corners, &i_list, 0.09);
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
        let r_step = compute_r_step(&tail_seq, &corners, &i_list, 0.09);
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
