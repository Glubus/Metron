use super::sums::{cumulative_sum, query_cumsum};

#[derive(Clone, Copy)]
pub enum SmoothMode { Avg, Sum }

#[inline]
pub fn smooth_on_corners(x: &[f64], f: &[f64], window: f64, scale: f64, mode: SmoothMode) -> Vec<f64> {
    let f_cumsum = cumulative_sum(x, f);
    let mut g = vec![0.0; f.len()];
    for (i, &s) in x.iter().enumerate() {
        let a = (s - window).max(x[0]);
        let b = (s + window).min(*x.last().expect("non-empty"));
        let val = query_cumsum(b, x, &f_cumsum, f) - query_cumsum(a, x, &f_cumsum, f);
        g[i] = match mode {
            SmoothMode::Avg => { if (b - a) > 0.0 { val / (b - a) } else { 0.0 } }
            SmoothMode::Sum => scale * val,
        };
    }
    g
}

#[inline]
pub fn rescale_high(sr: f64) -> f64 {
    if sr <= 9.0 { return sr; }
    9.0 + (sr - 9.0) * (1.0 / 1.2)
}
