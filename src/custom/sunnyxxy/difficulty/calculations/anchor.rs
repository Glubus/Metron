/// `key_usage_400` is flat col-major: index = col * n + idx
pub fn compute_anchor_into(
    k: usize,
    key_usage_400: &[f64],
    n: usize,
    base_corners: &[f64],
    out: &mut Vec<f64>,
) {
    out.resize(n, 0.0);
    let mut counts = [0.0f64; 10]; // k ≤ 10
    for idx in 0..n {
        let mut cnt = 0usize;
        for col in 0..k {
            let v = key_usage_400[col * n + idx];
            if v != 0.0 { counts[cnt] = v; cnt += 1; }
        }
        if cnt > 1 {
            counts[..cnt].sort_unstable_by(|a, b| b.partial_cmp(a).expect("finite"));
            let mut walk = 0.0;
            let mut max_walk = 0.0;
            for i in 0..(cnt - 1) {
                let a = counts[i];
                let b = counts[i + 1];
                let term = a * (1.0 - 4.0 * (0.5 - b / a).powi(2));
                walk += term;
                max_walk += a;
            }
            out[idx] = if max_walk.abs() > 0.0 { walk / max_walk } else { 0.0 };
        } else {
            out[idx] = 0.0;
        }
    }
    for v in out.iter_mut() {
        let a = *v - 0.18;
        let b = 5.0 * (*v - 0.22).powi(3);
        *v = 1.0 + a.min(b);
    }
    let _ = base_corners;
}
