use super::note::Note;

pub fn find_next_note_in_column(
    note: (usize, i64, i64),
    times: &[i64],
    notes_by_column: &[Vec<Note>],
) -> (usize, i64, i64) {
    let (k, h, _t) = note;
    let idx = times.partition_point(|&val| val < h);
    if idx + 1 < notes_by_column[k].len() {
        let next_note = &notes_by_column[k][idx + 1];
        (next_note.column, next_note.hit_time, next_note.tail_time)
    } else {
        (0, 1_000_000_000, 1_000_000_000)
    }
}
