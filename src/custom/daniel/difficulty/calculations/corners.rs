use super::super::note::Note;

pub fn get_corners(total_duration: i64, notes: &[Note]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut base = Vec::with_capacity(notes.len() * 4 + 2);
    for note in notes {
        let h = note.hit_time;
        base.push(h);
        base.push(h + 501);
        base.push(h - 499);
        base.push(h + 1);
    }
    base.push(0);
    base.push(total_duration);
    retain_sort_dedup_in_range(&mut base, 0, total_duration);

    let mut a_corners = Vec::with_capacity(notes.len() * 3 + 2);
    for note in notes {
        let h = note.hit_time;
        a_corners.push(h);
        a_corners.push(h + 1000);
        a_corners.push(h - 1000);
    }
    a_corners.push(0);
    a_corners.push(total_duration);
    retain_sort_dedup_in_range(&mut a_corners, 0, total_duration);

    let mut all_corners = base.clone();
    all_corners.extend_from_slice(&a_corners);
    all_corners.sort_unstable();
    all_corners.dedup();

    (
        all_corners.into_iter().map(|v| v as f64).collect(),
        base.into_iter().map(|v| v as f64).collect(),
        a_corners.into_iter().map(|v| v as f64).collect(),
    )
}

fn retain_sort_dedup_in_range(values: &mut Vec<i64>, min_value: i64, max_value: i64) {
    values.retain(|&value| (min_value..=max_value).contains(&value));
    values.sort_unstable();
    values.dedup();
}
