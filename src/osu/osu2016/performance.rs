use super::super::osu2018::difficulty::Osu2018Difficulty;
use super::Osu2016PerformanceContext;
use crate::calculator::Rating;
use rox::RoxChart;

#[derive(Debug)]
pub struct Osu2016Performance {
    pub pp: f64,
    pub strain_value: f64,
    pub acc_value: f64,
}

impl Rating for Osu2016Performance {}

/// Calculate the performance of a beatmap.
///
/// # Errors
///
/// This function will return an error if the difficulty calculation fails.
pub fn calculate(
    _chart: &RoxChart,
    diff: &Osu2018Difficulty,
    context: &Osu2016PerformanceContext,
) -> crate::calculator::CalculatorResult<Osu2016Performance> {
    let strain_base = (5.0 * (diff.stars / 0.0825).max(1.0) - 4.0).powi(3) / 110_000.0
        * (1.0 + 0.1 * (f64::from(diff.object_count) / 1500.0).min(1.0));

    let score = context.score;

    let strain_multiplier = if score < 500_000.0 {
        score / 500_000.0 * 0.1
    } else if score < 600_000.0 {
        (score - 500_000.0) / 100_000.0 * 0.3
    } else if score < 700_000.0 {
        (score - 600_000.0) / 100_000.0 * 0.35 + 0.3
    } else if score < 800_000.0 {
        (score - 700_000.0) / 100_000.0 * 0.2 + 0.65
    } else if score < 900_000.0 {
        (score - 800_000.0) / 100_000.0 * 0.1 + 0.85
    } else {
        (score - 900_000.0) / 100_000.0 * 0.05 + 0.95
    };

    let od_window = diff.great_hit_window;
    let acc_value = ((150.0 / od_window) * context.accuracy.powi(16)).powf(1.8)
        * 2.5
        * (f64::from(diff.object_count) / 1500.0).powf(0.3).min(1.15);
    // nerfpp assumed 1.0 from plan
    let total_pp =
        (acc_value.powf(1.1) + (strain_base * strain_multiplier).powf(1.1)).powf(1.0 / 1.1) * 1.1;

    Ok(Osu2016Performance {
        pp: total_pp,
        strain_value: strain_base * strain_multiplier,
        acc_value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        clock_rate::ClockRate,
        osu::osu2018::{difficulty::calculate as osu2018_calculate, Osu2018DifficultyContext},
    };
    use approx::assert_abs_diff_eq;
    use rox::RoxChart;
    use rox_formats::auto_decode;
    use rstest::{fixture, rstest};

    #[fixture]
    fn chart() -> RoxChart {
        auto_decode("assets/test.osu").expect("Failed to decode test.osu")
    }

    #[rstest]
    fn test_osu2016_performance_calculation(chart: RoxChart) {
        let context = Osu2018DifficultyContext {
            clock_rate: None,
            overall_difficulty: Some(8.0),
        };
        let diff = osu2018_calculate(&chart, &context).expect("Difficulty calculation failed");

        let context = Osu2016PerformanceContext {
            accuracy: 1.0,
            score: 1_000_000.0,
        };
        let result = calculate(&chart, &diff, &context).expect("Performance calculation failed");

        println!("Calculated PP: {}", result.pp);
        assert_abs_diff_eq!(result.pp, 686.559_458_054_166_7, epsilon = 0.001);
    }

    #[rstest]
    fn test_osu2016_performance_calculation_900000_score(chart: RoxChart) {
        let context = Osu2018DifficultyContext {
            clock_rate: None,
            overall_difficulty: Some(8.0),
        };
        let diff = osu2018_calculate(&chart, &context).expect("Difficulty calculation failed");

        let context = Osu2016PerformanceContext {
            accuracy: 0.98,
            score: 900_000.0,
        };
        let result = calculate(&chart, &diff, &context).expect("Performance calculation failed");

        println!("Calculated PP: {}", result.pp);
        assert_abs_diff_eq!(result.pp, 642.567_965_400_809_5, epsilon = 0.001);
    }

    #[rstest]
    fn test_osu2016_performance_calculation_jakads_2016(chart: RoxChart) {
        let context = Osu2018DifficultyContext {
            clock_rate: None,
            overall_difficulty: Some(8.0),
        };
        let diff = osu2018_calculate(&chart, &context).expect("Difficulty calculation failed");

        let context = Osu2016PerformanceContext {
            accuracy: 0.9986,
            score: 994_346.0,
        };
        let result = calculate(&chart, &diff, &context).expect("Performance calculation failed");

        println!("Calculated PP: {}", result.pp);
        assert_abs_diff_eq!(result.pp, 683.691_622_322_474_5, epsilon = 0.001);
    }

    #[rstest]
    fn test_osu2016_performance_calculation_dt_score(chart: RoxChart) {
        let context = Osu2018DifficultyContext {
            clock_rate: Some(ClockRate::from_percentage(150).expect("Valid clock rate")),
            overall_difficulty: Some(8.0),
        };
        let diff = osu2018_calculate(&chart, &context).expect("Difficulty calculation failed");

        let context = Osu2016PerformanceContext {
            accuracy: 0.9212,
            score: 737_120.0,
        };
        let result = calculate(&chart, &diff, &context).expect("Performance calculation failed");

        println!("Calculated PP: {}", result.pp);
        assert_abs_diff_eq!(result.pp, 1_247.581_468_145_209_3, epsilon = 0.001);
    }
}
