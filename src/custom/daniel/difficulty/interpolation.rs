pub fn interp_values(new_x: &[f64], old_x: &[f64], old_vals: &[f64]) -> Vec<f64> {
    let mut result = Vec::with_capacity(new_x.len());
    interp_values_into(new_x, old_x, old_vals, &mut result);
    result
}

pub fn interp_values_into(new_x: &[f64], old_x: &[f64], old_vals: &[f64], out: &mut Vec<f64>) {
    out.clear();
    let mut ptr = 0usize;

    for &value in new_x {
        while ptr + 1 < old_x.len() && old_x[ptr + 1] <= value {
            ptr += 1;
        }
        if value < old_x[0] {
            out.push(old_vals[0]);
        } else if value >= old_x[old_x.len() - 1] {
            out.push(old_vals[old_vals.len() - 1]);
        } else {
            let x0 = old_x[ptr];
            let x1 = old_x[ptr + 1];
            let t = (value - x0) / (x1 - x0);
            out.push(old_vals[ptr] + t * (old_vals[ptr + 1] - old_vals[ptr]));
        }
    }
}

pub fn step_interp(new_x: &[f64], old_x: &[f64], old_vals: &[f64]) -> Vec<f64> {
    let mut result = Vec::with_capacity(new_x.len());
    step_interp_into(new_x, old_x, old_vals, &mut result);
    result
}

pub fn step_interp_into(new_x: &[f64], old_x: &[f64], old_vals: &[f64], out: &mut Vec<f64>) {
    out.clear();
    let mut ptr = 0usize;

    for &value in new_x {
        while ptr + 1 < old_x.len() && old_x[ptr + 1] <= value {
            ptr += 1;
        }
        out.push(old_vals[ptr]);
    }
}

pub fn search_left(values: &[f64], target: f64) -> usize {
    values.partition_point(|&value| value < target)
}

pub fn search_right(values: &[f64], target: f64) -> usize {
    values.partition_point(|&value| value <= target)
}
