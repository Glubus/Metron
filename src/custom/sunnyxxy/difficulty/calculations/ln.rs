use super::super::note::Note;
use std::collections::HashMap;

#[inline]
pub fn ln_bodies_count_sparse_representation(
    long_notes: &[Note],
    t: i64,
) -> (Vec<i64>, Vec<f64>, Vec<f64>) {
    let mut diff: HashMap<i64, f64> = HashMap::new();
    for note in long_notes.iter() {
        let t0 = (note.hit_time + 60).min(note.tail_time);
        let t1 = (note.hit_time + 120).min(note.tail_time);
        *diff.entry(t0).or_insert(0.0) += 1.3;
        *diff.entry(t1).or_insert(0.0) += -1.3 + 1.0;
        *diff.entry(note.tail_time).or_insert(0.0) += -1.0;
    }
    let mut points: Vec<i64> = diff.keys().cloned().collect();
    points.push(0);
    points.push(t);
    points.sort_unstable();
    points.dedup();

    let mut values: Vec<f64> = Vec::with_capacity(points.len().saturating_sub(1));
    let mut cumsum: Vec<f64> = Vec::with_capacity(points.len());
    cumsum.push(0.0);
    let mut curr = 0.0;
    for i in 0..(points.len().saturating_sub(1)) {
        let t = points[i];
        if let Some(dv) = diff.get(&t) {
            curr += *dv;
        }
        let v = curr.min(2.5 + 0.5 * curr);
        values.push(v);
        let seg_length = (points[i + 1] - points[i]) as f64;
        let last = *cumsum.last().expect("non-empty");
        cumsum.push(last + seg_length * v);
    }
    (points, cumsum, values)
}

#[inline]
pub fn ln_sum(a: f64, b: f64, ln_rep: &(Vec<i64>, Vec<f64>, Vec<f64>)) -> f64 {
    let (points, cumsum, values) = ln_rep;
    let i = points.partition_point(|&p| (p as f64) <= a);
    let i = if i == 0 { 0usize } else { i - 1 };
    let j = points.partition_point(|&p| (p as f64) <= b);
    let j = if j == 0 { 0usize } else { j - 1 };
    let mut total = 0.0;
    if i == j {
        total = (b - a) * values[i];
    } else {
        total += (points[i + 1] as f64 - a) * values[i];
        if j > i + 1 {
            total += cumsum[j] - cumsum[i + 1];
        }
        total += (b - points[j] as f64) * values[j];
    }
    total
}
