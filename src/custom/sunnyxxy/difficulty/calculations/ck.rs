use super::super::note::Note;

pub fn compute_c_and_ks(
    _k: usize,
    _t: i64,
    notes: &[Note],
    active_mask: &[u16],
    base_corners: &[f64],
    c_out: &mut Vec<f64>,
    ks_out: &mut Vec<f64>,
) {
    // notes are already sorted by hit_time (done in chart_to_notes)
    let n = base_corners.len();
    let nt = notes.len();
    c_out.resize(n, 0.0);
    c_out[..n].fill(0.0);
    let mut lp = 0usize;
    let mut rp = 0usize;
    for (i, &s) in base_corners.iter().enumerate() {
        let low = s - 500.0;
        let high = s + 500.0;
        while rp < nt && (notes[rp].hit_time as f64) < high { rp += 1; }
        while lp < nt && (notes[lp].hit_time as f64) < low { lp += 1; }
        c_out[i] = (rp - lp) as f64;
    }
    ks_out.resize(n, 0.0);
    for i in 0..n {
        ks_out[i] = (active_mask[i].count_ones().max(1)) as f64;
    }
}
