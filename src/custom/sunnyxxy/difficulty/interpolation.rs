pub fn interp_values(new_x: &[f64], old_x: &[f64], old_vals: &[f64]) -> Vec<f64> {
    let mut out = Vec::with_capacity(new_x.len());
    interp_values_into(new_x, old_x, old_vals, &mut out);
    out
}

pub fn interp_values_into(new_x: &[f64], old_x: &[f64], old_vals: &[f64], out: &mut Vec<f64>) {
    let m = old_x.len();
    out.clear();
    let mut ptr = 0usize;
    for &nx in new_x.iter() {
        while ptr + 1 < m && old_x[ptr + 1] <= nx { ptr += 1; }
        if nx < old_x[0] {
            out.push(old_vals[0]);
        } else if nx >= old_x[m - 1] {
            out.push(old_vals[m - 1]);
        } else {
            let x0 = old_x[ptr];
            let x1 = old_x[ptr + 1];
            let t = (nx - x0) / (x1 - x0);
            out.push(old_vals[ptr] + t * (old_vals[ptr + 1] - old_vals[ptr]));
        }
    }
}

pub fn step_interp(new_x: &[f64], old_x: &[f64], old_vals: &[f64]) -> Vec<f64> {
    let mut out = Vec::with_capacity(new_x.len());
    step_interp_into(new_x, old_x, old_vals, &mut out);
    out
}

pub fn step_interp_into(new_x: &[f64], old_x: &[f64], old_vals: &[f64], out: &mut Vec<f64>) {
    let m = old_x.len();
    out.clear();
    let mut ptr = 0usize;
    for &nx in new_x.iter() {
        while ptr + 1 < m && old_x[ptr + 1] <= nx { ptr += 1; }
        out.push(old_vals[ptr]);
    }
}
