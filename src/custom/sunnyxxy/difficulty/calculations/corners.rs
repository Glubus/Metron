use std::cell::RefCell;
use super::super::note::Note;

thread_local! {
    static CORNERS_BASE_I64: RefCell<Vec<i64>> = RefCell::new(Vec::new());
    static CORNERS_A_I64: RefCell<Vec<i64>> = RefCell::new(Vec::new());
    static CORNERS_ALL_I64: RefCell<Vec<i64>> = RefCell::new(Vec::new());
}

pub fn get_corners_into(t: i64, notes: &[Note], all_corners: &mut Vec<f64>, base_corners: &mut Vec<f64>, a_corners: &mut Vec<f64>) {
    CORNERS_BASE_I64.with(|bc_cell| {
        CORNERS_A_I64.with(|ac_cell| {
            CORNERS_ALL_I64.with(|all_cell| {
                let mut base_cands = bc_cell.borrow_mut();
                let mut a_cands = ac_cell.borrow_mut();
                let mut all_cands = all_cell.borrow_mut();

                base_cands.clear();
                for note in notes.iter() {
                    base_cands.push(note.hit_time);
                    if note.tail_time >= 0 { base_cands.push(note.tail_time); }
                }
                let snapshot_len = base_cands.len();
                for i in 0..snapshot_len {
                    let s = base_cands[i];
                    base_cands.push(s + 501);
                    base_cands.push(s - 499);
                    base_cands.push(s + 1);
                }
                base_cands.push(0);
                base_cands.push(t);
                base_cands.retain(|&s| 0 <= s && s <= t);
                base_cands.sort_unstable();
                base_cands.dedup();

                a_cands.clear();
                for note in notes.iter() {
                    a_cands.push(note.hit_time);
                    if note.tail_time >= 0 { a_cands.push(note.tail_time); }
                }
                let snapshot_a_len = a_cands.len();
                for i in 0..snapshot_a_len {
                    let s = a_cands[i];
                    a_cands.push(s + 1000);
                    a_cands.push(s - 1000);
                }
                a_cands.push(0);
                a_cands.push(t);
                a_cands.retain(|&s| 0 <= s && s <= t);
                a_cands.sort_unstable();
                a_cands.dedup();

                all_cands.clear();
                all_cands.extend_from_slice(&base_cands);
                all_cands.extend_from_slice(&a_cands);
                all_cands.sort_unstable();
                all_cands.dedup();

                all_corners.clear();
                all_corners.extend(all_cands.iter().map(|&v| v as f64));
                base_corners.clear();
                base_corners.extend(base_cands.iter().map(|&v| v as f64));
                a_corners.clear();
                a_corners.extend(a_cands.iter().map(|&v| v as f64));
            })
        })
    })
}
