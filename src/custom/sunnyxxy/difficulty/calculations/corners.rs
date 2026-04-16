use super::super::note::Note;

pub fn get_corners(t: i64, notes: &[Note]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut base_candidates: Vec<i64> = Vec::with_capacity(notes.len() * 4 + 4);
    for note in notes.iter() {
        base_candidates.push(note.hit_time);
        if note.tail_time >= 0 {
            base_candidates.push(note.tail_time);
        }
    }
    let snapshot_len = base_candidates.len();
    for i in 0..snapshot_len {
        let s = base_candidates[i];
        base_candidates.push(s + 501);
        base_candidates.push(s - 499);
        base_candidates.push(s + 1);
    }
    base_candidates.push(0);
    base_candidates.push(t);
    base_candidates.retain(|&s| 0 <= s && s <= t);
    base_candidates.sort_unstable();
    base_candidates.dedup();
    let corners_base_vec = base_candidates;

    let mut a_candidates: Vec<i64> = Vec::with_capacity(notes.len() * 3 + 2);
    for note in notes.iter() {
        a_candidates.push(note.hit_time);
        if note.tail_time >= 0 {
            a_candidates.push(note.tail_time);
        }
    }
    let snapshot_a_len = a_candidates.len();
    for i in 0..snapshot_a_len {
        let s = a_candidates[i];
        a_candidates.push(s + 1000);
        a_candidates.push(s - 1000);
    }
    a_candidates.push(0);
    a_candidates.push(t);
    a_candidates.retain(|&s| 0 <= s && s <= t);
    a_candidates.sort_unstable();
    a_candidates.dedup();
    let corners_a_vec = a_candidates;

    let mut all_corners: Vec<i64> = Vec::with_capacity(corners_base_vec.len() + corners_a_vec.len());
    all_corners.extend_from_slice(&corners_base_vec);
    all_corners.extend_from_slice(&corners_a_vec);
    all_corners.sort_unstable();
    all_corners.dedup();

    (
        all_corners.into_iter().map(|v| v as f64).collect(),
        corners_base_vec.into_iter().map(|v| v as f64).collect(),
        corners_a_vec.into_iter().map(|v| v as f64).collect(),
    )
}
