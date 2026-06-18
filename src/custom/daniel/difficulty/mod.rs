pub mod bars;
pub mod calculations;
pub mod interpolation;
pub mod map_data;
pub mod metrics;
pub mod note;
pub mod process;
pub mod smoothing;

use rhythm_open_exchange::RoxChart;

use crate::calculator::{CalculatorResult, Rating};

use super::DanielDifficultyContext;

pub use metrics::{DanielFactorAverages, factor_averages};

#[derive(Debug, Clone)]
pub struct DanielDifficultyGraph {
    pub times_ms: Vec<f64>,
    pub values: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct DanielFactorCurves {
    pub pressing_intensity: Vec<f64>,
    pub unevenness: Vec<f64>,
    pub same_column_pressure: Vec<f64>,
    pub cross_column_pressure: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct DanielDifficulty {
    pub stars: f64,
    pub graph: DanielDifficultyGraph,
    pub factors: DanielFactorCurves,
}

impl DanielDifficulty {
    #[must_use]
    pub fn factor_averages(&self) -> DanielFactorAverages {
        factor_averages(&self.graph.times_ms, &self.factors)
    }
}

impl Rating for DanielDifficulty {}

pub fn calculate(
    chart: &RoxChart,
    context: &DanielDifficultyContext,
) -> CalculatorResult<DanielDifficulty> {
    process::calculate(chart, context)
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use rhythm_open_exchange::RoxChart;
    use rox_formats::auto_decode;

    use crate::calculator::Calculator;
    use crate::clock_rate::ClockRate;

    use super::super::Daniel;
    use super::*;

    #[test]
    fn test_compute_hit_leniency_matches_reference() {
        let x = map_data::compute_hit_leniency(8.0);
        let expected = 0.3 * ((64.5 - 24.0) / 500.0_f64).sqrt();
        assert_abs_diff_eq!(x, expected, epsilon = 1e-12);
    }

    #[test]
    fn test_factor_averages_constant_curve() {
        let times = [0.0, 100.0, 200.0];
        let factors = DanielFactorCurves {
            pressing_intensity: vec![2.0, 2.0, 2.0],
            unevenness: vec![1.0, 1.0, 1.0],
            same_column_pressure: vec![3.0, 3.0, 3.0],
            cross_column_pressure: vec![4.0, 4.0, 4.0],
        };

        let averages = factor_averages(&times, &factors);
        assert_eq!(
            averages,
            DanielFactorAverages {
                pressing_intensity: 2.0,
                unevenness: 1.0,
                same_column_pressure: 3.0,
                cross_column_pressure: 4.0,
            }
        );
    }

    fn test_chart() -> RoxChart {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/test.osu");
        auto_decode(path).expect("Failed to decode test.osu")
    }

    #[test]
    fn test_daniel_calculates_non_empty_outputs() {
        let chart = test_chart();
        let calc = Daniel;
        let difficulty = calc
            .calculate_difficulty(&chart, &super::super::DanielDifficultyContext::default())
            .expect("Daniel calculation should succeed");

        assert!(difficulty.stars.is_finite());
        assert!(difficulty.stars > 0.0);
        assert_eq!(
            difficulty.graph.times_ms.len(),
            difficulty.graph.values.len()
        );
        assert_eq!(
            difficulty.graph.times_ms.len(),
            difficulty.factors.pressing_intensity.len()
        );
        assert_eq!(
            difficulty.graph.times_ms.len(),
            difficulty.factors.unevenness.len()
        );
        assert_eq!(
            difficulty.graph.times_ms.len(),
            difficulty.factors.same_column_pressure.len()
        );
        assert_eq!(
            difficulty.graph.times_ms.len(),
            difficulty.factors.cross_column_pressure.len()
        );
    }

    #[test]
    fn test_daniel_rate_change_affects_stars() {
        let chart = test_chart();
        let calc = Daniel;
        let normal = calc
            .calculate_difficulty(&chart, &super::super::DanielDifficultyContext::default())
            .expect("Daniel calculation should succeed");
        let faster = calc
            .calculate_difficulty(
                &chart,
                &super::super::DanielDifficultyContext {
                    clock_rate: Some(ClockRate::from_percentage(150).expect("valid rate")),
                    overall_difficulty: None,
                },
            )
            .expect("Daniel calculation should succeed");

        assert!(faster.stars > normal.stars);
    }

    #[test]
    fn test_daniel_golden_star_values() {
        let chart = test_chart();
        let calc = Daniel;

        let normal = calc
            .calculate_difficulty(&chart, &super::super::DanielDifficultyContext::default())
            .expect("Daniel calculation should succeed");
        let dt = calc
            .calculate_difficulty(
                &chart,
                &super::super::DanielDifficultyContext {
                    clock_rate: Some(ClockRate::from_percentage(150).expect("valid rate")),
                    overall_difficulty: None,
                },
            )
            .expect("Daniel calculation should succeed");

        assert_abs_diff_eq!(normal.stars, 4.694_315_670_550_038, epsilon = 1e-6);
        assert_abs_diff_eq!(dt.stars, 7.283_613_227_167_455, epsilon = 1e-6);
    }
}
