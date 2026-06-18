#[derive(Clone, Copy)]
pub enum SmoothMode {
    Avg,
    Sum,
}

pub fn smooth_on_corners(
    x: &[f64],
    f: &[f64],
    window: f64,
    scale: f64,
    mode: SmoothMode,
) -> Vec<f64> {
    let mut result = vec![0.0; x.len()];
    smooth_on_corners_into(x, f, window, scale, mode, &mut result);
    result
}

pub fn smooth_on_corners_into(
    x: &[f64],
    f: &[f64],
    window: f64,
    scale: f64,
    mode: SmoothMode,
    out: &mut [f64],
) {
    let cumulative = cumulative_sum(x, f);
    let x_min = x[0];
    let x_max = *x.last().expect("non-empty");

    for (index, &point) in x.iter().enumerate() {
        let a = (point - window).clamp(x_min, x_max);
        let b = (point + window).clamp(x_min, x_max);
        let value = query_piecewise_integral(x, f, &cumulative, b)
            - query_piecewise_integral(x, f, &cumulative, a);
        match mode {
            SmoothMode::Avg => {
                let span = b - a;
                out[index] = if span > 0.0 { value / span } else { 0.0 };
            }
            SmoothMode::Sum => out[index] = scale * value,
        }
    }
}

pub fn gaussian_filter1d(data: &[f64], sigma: f64) -> Vec<f64> {
    if data.is_empty() {
        return Vec::new();
    }

    let kernel_radius = (4.0 * sigma + 0.5) as usize;
    let mut kernel = Vec::with_capacity(kernel_radius * 2 + 1);
    for i in 0..=(kernel_radius * 2) {
        let x = i as isize - kernel_radius as isize;
        kernel.push((-0.5 * ((x as f64) / sigma).powi(2)).exp());
    }
    let kernel_sum: f64 = kernel.iter().sum();
    for value in &mut kernel {
        *value /= kernel_sum;
    }

    let mut padded = vec![0.0; kernel_radius];
    padded.extend_from_slice(data);
    padded.extend(std::iter::repeat_n(0.0, kernel_radius));

    let mut result = vec![0.0; data.len()];
    for i in 0..data.len() {
        let mut sum = 0.0;
        for (j, &weight) in kernel.iter().enumerate() {
            sum += padded[i + j] * weight;
        }
        result[i] = sum;
    }
    result
}

pub fn rescale_high(stars: f64) -> f64 {
    if stars <= 9.0 {
        stars
    } else {
        9.0 + (stars - 9.0) / 1.2
    }
}

fn cumulative_sum(x: &[f64], f: &[f64]) -> Vec<f64> {
    let mut cumulative = vec![0.0; x.len()];
    for i in 1..x.len() {
        cumulative[i] = cumulative[i - 1] + f[i - 1] * (x[i] - x[i - 1]);
    }
    cumulative
}

fn query_piecewise_integral(x: &[f64], f: &[f64], cumulative: &[f64], q: f64) -> f64 {
    let idx = crate::custom::daniel::difficulty::interpolation::search_left(x, q)
        .saturating_sub(1)
        .min(x.len() - 2);
    cumulative[idx] + f[idx] * (q - x[idx])
}
