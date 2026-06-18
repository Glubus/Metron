use super::super::interpolation::search_left;
use super::super::note::Note;

pub fn compute_c_and_ks(
    notes: &[Note],
    active_masks: &[u16],
    base_corners: &[f64],
) -> (Vec<f64>, Vec<f64>) {
    let note_hit_times: Vec<f64> = notes.iter().map(|note| note.hit_time as f64).collect();

    let mut c_step = vec![0.0; base_corners.len()];
    let mut ks_step = vec![0.0; base_corners.len()];

    for (i, &corner) in base_corners.iter().enumerate() {
        let lo = search_left(&note_hit_times, corner - 500.0);
        let hi = search_left(&note_hit_times, corner + 500.0);
        c_step[i] = (hi - lo) as f64;
        ks_step[i] = active_masks[i].count_ones().max(1) as f64;
    }

    (c_step, ks_step)
}
