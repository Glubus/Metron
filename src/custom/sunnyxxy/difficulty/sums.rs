#[inline]
pub fn cumulative_sum(x: &[f64], f: &[f64]) -> Vec<f64> {
    let mut f_cumsum = vec![0.0; x.len()];
    for i in 1..x.len() {
        f_cumsum[i] = f_cumsum[i - 1] + f[i - 1] * (x[i] - x[i - 1]);
    }
    f_cumsum
}

#[inline]
pub fn query_cumsum(q: f64, x: &[f64], f_cumsum: &[f64], f: &[f64]) -> f64 {
    if q <= x[0] { return 0.0; }
    if q >= *x.last().expect("non-empty") {
        return *f_cumsum.last().expect("non-empty");
    }
    let i = x.partition_point(|&val| val <= q);
    let idx = if i == 0 { 0 } else { i - 1 };
    f_cumsum[idx] + f[idx] * (q - x[idx])
}
