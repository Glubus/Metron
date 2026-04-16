use super::super::note::Note;
use std::collections::HashMap;

pub fn compute_c_and_ks(
    k: usize,
    _t: i64,
    notes: &[Note],
    key_usage: &HashMap<usize, Vec<bool>>,
    base_corners: &[f64],
) -> (Vec<f64>, Vec<f64>) {
    let mut note_hit_times: Vec<i64> = notes.iter().map(|note| note.hit_time).collect();
    note_hit_times.sort_unstable();
    let n = base_corners.len();
    let mut c_step = vec![0.0; n];
    for (i, &s) in base_corners.iter().enumerate() {
        let low = s - 500.0;
        let high = s + 500.0;
        let left_high = note_hit_times.partition_point(|&t| (t as f64) < high);
        let left_low = note_hit_times.partition_point(|&t| (t as f64) < low);
        let cnt = (left_high as i64 - left_low as i64) as f64;
        c_step[i] = cnt;
    }
    let mut ks_step = vec![0.0; n];
    for i in 0..n {
        let mut cnt = 0usize;
        for col in 0..k {
            if key_usage.get(&col).map(|v| v[i]).unwrap_or(false) {
                cnt += 1;
            }
        }
        ks_step[i] = (cnt.max(1)) as f64;
    }
    (c_step, ks_step)
}
