pub fn interp_values(new_x: &[f64], old_x: &[f64], old_vals: &[f64]) -> Vec<f64> {
    let mut new_vals = Vec::with_capacity(new_x.len());
    for &nx in new_x.iter() {
        let idx = old_x.partition_point(|&ox| ox <= nx);
        if idx == 0 {
            new_vals.push(old_vals[0]);
        } else if idx >= old_x.len() {
            new_vals.push(*old_vals.last().expect("non-empty"));
        } else {
            let x0 = old_x[idx - 1];
            let x1 = old_x[idx];
            let y0 = old_vals[idx - 1];
            let y1 = old_vals[idx];
            let t = (nx - x0) / (x1 - x0);
            new_vals.push(y0 + t * (y1 - y0));
        }
    }
    new_vals
}

pub fn step_interp(new_x: &[f64], old_x: &[f64], old_vals: &[f64]) -> Vec<f64> {
    let mut res = Vec::with_capacity(new_x.len());
    for &nx in new_x.iter() {
        let mut idx = old_x.partition_point(|&ox| ox <= nx);
        if idx > 0 { idx -= 1; }
        if idx >= old_vals.len() { idx = old_vals.len() - 1; }
        res.push(old_vals[idx]);
    }
    res
}
