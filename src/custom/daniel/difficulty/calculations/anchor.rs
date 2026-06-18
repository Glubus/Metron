use std::cmp::Ordering;

pub fn compute_anchor(key_count: usize, key_usage_400: &[f64], corner_count: usize) -> Vec<f64> {
    let mut anchor = vec![1.0; corner_count];

    for i in 0..corner_count {
        let mut counts = Vec::with_capacity(key_count);
        for column in 0..key_count {
            counts.push(key_usage_400[column * corner_count + i]);
        }
        counts.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        let nonzero_count = counts.iter().filter(|&&value| value > 0.0).count();
        let mut walk = 0.0;
        let mut max_walk = 0.0;

        for pair in counts.windows(2) {
            let c0 = pair[0];
            let c1 = pair[1];
            if c0 > 0.0 && c1 > 0.0 {
                let ratio = c1 / c0;
                let weight = 1.0 - 4.0 * (0.5 - ratio).powi(2);
                walk += c0 * weight;
                max_walk += c0;
            }
        }

        let raw_anchor = if nonzero_count > 1 {
            walk / max_walk.max(1e-9)
        } else {
            0.0
        };

        anchor[i] = 1.0 + (raw_anchor - 0.18).min(5.0 * (raw_anchor - 0.22).powi(3));
    }

    anchor
}
