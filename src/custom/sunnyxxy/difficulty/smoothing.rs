use std::cell::RefCell;

#[derive(Clone, Copy)]
pub enum SmoothMode { Avg, Sum }

thread_local! {
    static CUMSUM_SCRATCH: RefCell<Vec<f64>> = RefCell::new(Vec::new());
}

#[inline]
pub fn smooth_on_corners(x: &[f64], f: &[f64], window: f64, scale: f64, mode: SmoothMode) -> Vec<f64> {
    let n = x.len();
    CUMSUM_SCRATCH.with(|cell| {
        let mut scratch = cell.borrow_mut();
        if scratch.len() < n { scratch.resize(n, 0.0); }
        scratch[0] = 0.0;
        for i in 1..n {
            scratch[i] = scratch[i - 1] + f[i - 1] * (x[i] - x[i - 1]);
        }
        let mut g = vec![0.0; n];
        let x_min = x[0];
        let x_max = *x.last().expect("non-empty");
        let cum_max = scratch[n - 1];
        let mut lp = 0usize;
        let mut rp = 0usize;
        for (i, &s) in x.iter().enumerate() {
            let a = (s - window).max(x_min);
            let b = (s + window).min(x_max);
            while lp + 1 < n && x[lp + 1] <= a { lp += 1; }
            while rp + 1 < n && x[rp + 1] <= b { rp += 1; }
            let val_a = scratch[lp] + f[lp] * (a - x[lp]);
            let val_b = if b >= x_max { cum_max } else { scratch[rp] + f[rp] * (b - x[rp]) };
            let val = val_b - val_a;
            g[i] = match mode {
                SmoothMode::Avg => if (b - a) > 0.0 { val / (b - a) } else { 0.0 },
                SmoothMode::Sum => scale * val,
            };
        }
        g
    })
}

/// Like `smooth_on_corners` but writes into a pre-allocated mutable slice.
#[inline]
pub fn smooth_on_corners_into(x: &[f64], f: &[f64], window: f64, scale: f64, mode: SmoothMode, out: &mut [f64]) {
    let n = x.len();
    CUMSUM_SCRATCH.with(|cell| {
        let mut scratch = cell.borrow_mut();
        if scratch.len() < n { scratch.resize(n, 0.0); }
        scratch[0] = 0.0;
        for i in 1..n {
            scratch[i] = scratch[i - 1] + f[i - 1] * (x[i] - x[i - 1]);
        }
        let x_min = x[0];
        let x_max = *x.last().expect("non-empty");
        let cum_max = scratch[n - 1];
        let mut lp = 0usize;
        let mut rp = 0usize;
        for (i, &s) in x.iter().enumerate() {
            let a = (s - window).max(x_min);
            let b = (s + window).min(x_max);
            while lp + 1 < n && x[lp + 1] <= a { lp += 1; }
            while rp + 1 < n && x[rp + 1] <= b { rp += 1; }
            let val_a = scratch[lp] + f[lp] * (a - x[lp]);
            let val_b = if b >= x_max { cum_max } else { scratch[rp] + f[rp] * (b - x[rp]) };
            let val = val_b - val_a;
            out[i] = match mode {
                SmoothMode::Avg => if (b - a) > 0.0 { val / (b - a) } else { 0.0 },
                SmoothMode::Sum => scale * val,
            };
        }
    })
}

#[inline]
pub fn rescale_high(sr: f64) -> f64 {
    if sr <= 9.0 { return sr; }
    9.0 + (sr - 9.0) * (1.0 / 1.2)
}
