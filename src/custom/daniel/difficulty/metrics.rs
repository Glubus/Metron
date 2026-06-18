use super::DanielFactorCurves;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DanielFactorAverages {
    pub pressing_intensity: f64,
    pub unevenness: f64,
    pub same_column_pressure: f64,
    pub cross_column_pressure: f64,
}

pub fn factor_averages(times: &[f64], factors: &DanielFactorCurves) -> DanielFactorAverages {
    let pressing_intensity = trapezoidal_average(times, &factors.pressing_intensity);
    let unevenness = trapezoidal_average(times, &factors.unevenness);
    let same_column_pressure = trapezoidal_average(times, &factors.same_column_pressure);
    let cross_column_pressure = trapezoidal_average(times, &factors.cross_column_pressure);

    DanielFactorAverages {
        pressing_intensity,
        unevenness,
        same_column_pressure,
        cross_column_pressure,
    }
}

fn trapezoidal_average(times: &[f64], values: &[f64]) -> f64 {
    if times.len() < 2 || values.len() < 2 {
        return values.first().copied().unwrap_or(0.0);
    }

    let mut integral = 0.0;
    for i in 1..times.len() {
        integral += (values[i - 1] + values[i]) * (times[i] - times[i - 1]) * 0.5;
    }

    let duration = times[times.len() - 1] - times[0];
    if duration > 0.0 {
        integral / duration
    } else {
        0.0
    }
}
