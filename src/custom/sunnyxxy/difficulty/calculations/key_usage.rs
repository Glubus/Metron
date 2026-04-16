use super::super::note::Note;
use std::collections::HashMap;

pub fn get_key_usage(
    k: usize,
    t: i64,
    notes: &[Note],
    base_corners: &[f64],
) -> HashMap<usize, Vec<bool>> {
    let mut key_usage: HashMap<usize, Vec<bool>> = HashMap::with_capacity(k);
    for col in 0..k {
        key_usage.insert(col, vec![false; base_corners.len()]);
    }
    for note in notes.iter() {
        let start_time = (note.hit_time - 150).max(0);
        let end_time = if note.tail_time < 0 {
            note.hit_time + 150
        } else {
            (note.tail_time + 150).min(t - 1)
        };
        let left_idx = base_corners.partition_point(|&v| v < start_time as f64);
        let right_idx = base_corners.partition_point(|&v| v < end_time as f64);
        if let Some(usage) = key_usage.get_mut(&note.column) {
            for i in left_idx..right_idx {
                usage[i] = true;
            }
        }
    }
    key_usage
}

pub fn get_key_usage_400(
    k: usize,
    t: i64,
    notes: &[Note],
    base_corners: &[f64],
) -> HashMap<usize, Vec<f64>> {
    let mut key_usage_400: HashMap<usize, Vec<f64>> = HashMap::new();
    for col in 0..k {
        key_usage_400.insert(col, vec![0.0; base_corners.len()]);
    }
    for note in notes.iter() {
        let start_time = note.hit_time.max(0);
        let end_time = if note.tail_time < 0 {
            note.hit_time
        } else {
            (note.tail_time).min(t - 1)
        };
        let left400_idx = base_corners.partition_point(|&v| v < (start_time - 400) as f64);
        let left_idx = base_corners.partition_point(|&v| v < start_time as f64);
        let right_idx = base_corners.partition_point(|&v| v < end_time as f64);
        let right400_idx = base_corners.partition_point(|&v| v < (end_time + 400) as f64);

        if let Some(usage) = key_usage_400.get_mut(&note.column) {
            for i in left_idx..right_idx {
                usage[i] += 3.75 + ((end_time - start_time).min(1500) as f64) / 150.0;
            }
            for i in left400_idx..left_idx {
                let diff = base_corners[i] - start_time as f64;
                usage[i] += 3.75 - 3.75 / (400.0 * 400.0) * diff * diff;
            }
            for i in right_idx..right400_idx {
                let diff = (base_corners[i] - end_time as f64).abs();
                usage[i] += 3.75 - 3.75 / (400.0 * 400.0) * diff * diff;
            }
        }
    }
    key_usage_400
}
