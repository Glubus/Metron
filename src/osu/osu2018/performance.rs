use super::difficulty::Osu2018Difficulty;
use super::Osu2018PerformanceContext;
use crate::calculator::Rating;
use rox::RoxChart;

#[derive(Debug)]
pub struct Osu2018Performance {
    pub pp: f64,
    pub strain_value: f64,
    pub acc_value: f64,
}

impl Rating for Osu2018Performance {}

/// Calculates the performance (PP) of a map.
///
/// # Errors
///
/// This function will return an error if the difficulty calculation fails.
pub fn calculate(
    _chart: &RoxChart,
    diff: &Osu2018Difficulty,
    context: &Osu2018PerformanceContext,
) -> crate::calculator::CalculatorResult<Osu2018Performance> {
    let score_multiplier = diff.score_multiplier;
    let raw_total_score = context.accuracy * 1_000_000.0;

    // Scale score up, comparable to keymods (C# logic)
    let scaled_score = f64::from(raw_total_score) * (1.0 / score_multiplier);

    // "Arbitrary initial value ... 0.8"
    let multiplier = 0.8;

    let strain_value = compute_strain_value(diff.stars, diff.object_count, scaled_score);
    let acc_value = compute_accuracy_value(diff.great_hit_window, strain_value, scaled_score);

    let total_value = (strain_value.powf(1.1) + acc_value.powf(1.1)).powf(1.0 / 1.1) * multiplier;

    Ok(Osu2018Performance {
        pp: total_value,
        strain_value,
        acc_value,
    })
}

#[allow(clippy::cast_precision_loss)]
fn compute_strain_value(stars: f64, total_hits: u32, scaled_score: f64) -> f64 {
    // Obtain strain difficulty
    // double strainValue = Math.Pow(5 * Math.Max(1, Attributes.StarRating / 0.2) - 4.0, 2.2) / 135.0;
    let mut strain_value = (5.0 * (stars / 0.2).max(1.0) - 4.0).powf(2.2) / 135.0;

    // Longer maps are worth more
    // strainValue *= 1.0 + 0.1 * Math.Min(1.0, totalHits / 1500.0);
    strain_value *= 1.0 + 0.1 * (f64::from(total_hits) / 1500.0).min(1.0);

    // Score penality
    if scaled_score <= 500_000.0 {
        strain_value = 0.0;
    } else if scaled_score <= 600_000.0 {
        strain_value *= (scaled_score - 500_000.0) / 100_000.0 * 0.3;
    } else if scaled_score <= 700_000.0 {
        strain_value *= 0.3 + (scaled_score - 600_000.0) / 100_000.0 * 0.25;
    } else if scaled_score <= 800_000.0 {
        strain_value *= 0.55 + (scaled_score - 700_000.0) / 100_000.0 * 0.20;
    } else if scaled_score <= 900_000.0 {
        strain_value *= 0.75 + (scaled_score - 800_000.0) / 100_000.0 * 0.15;
    } else {
        strain_value *= 0.90 + (scaled_score - 900_000.0) / 100_000.0 * 0.1;
    }

    strain_value
}

fn compute_accuracy_value(great_hit_window: f64, strain_value: f64, scaled_score: f64) -> f64 {
    if great_hit_window <= 0.0 {
        return 0.0;
    }

    // double accuracyValue = Math.Max(0.0, 0.2 - (Attributes.GreatHitWindow - 34) * 0.006667)
    //                        * strainValue
    //                        * Math.Pow(Math.Max(0.0, scaledScore - 960000) / 40000, 1.1);

    (0.2 - (great_hit_window - 34.0) * 0.006_667).max(0.0)
        * strain_value
        * ((scaled_score - 960_000.0).max(0.0) / 40_000.0).powf(1.1)
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
    fn test_osu2018_performance_calculation(chart: RoxChart) {
        let context = Osu2018DifficultyContext {
            clock_rate: None, // should be equal to 1.0
            overall_difficulty: Some(8.0),
        };
        let diff = osu2018_calculate(&chart, &context).expect("Difficulty calculation failed");
        let context = Osu2018PerformanceContext { accuracy: 1.0 };
        let result = calculate(&chart, &diff, &context).expect("Performance calculation failed");

        assert_abs_diff_eq!(result.pp, 511.857_268_058_640_56, epsilon = 0.001);
    }

    #[rstest]
    fn test_osu2018_performance_calculation_93_71(chart: RoxChart) {
        let context = Osu2018DifficultyContext {
            clock_rate: None,
            overall_difficulty: Some(8.0),
        };
        let diff = osu2018_calculate(&chart, &context).expect("Difficulty calculation failed");

        let context = Osu2018PerformanceContext { accuracy: 0.9371 };
        let result = calculate(&chart, &diff, &context).expect("Performance calculation failed");

        assert_abs_diff_eq!(result.pp, 428.117_399_809_329_8, epsilon = 0.001);
    }

    #[rstest]
    fn test_osu2018_performance_calculation_clock_rate(chart: RoxChart) {
        let context = Osu2018DifficultyContext {
            clock_rate: Some(ClockRate::from_percentage(200).expect("Valid clock rate")),
            overall_difficulty: Some(8.0),
        };
        let diff = osu2018_calculate(&chart, &context).expect("Difficulty calculation failed");

        let context = Osu2018PerformanceContext { accuracy: 0.90 };
        let result = calculate(&chart, &diff, &context).expect("Performance calculation failed");

        assert_abs_diff_eq!(result.pp, 1_407.589_999_202_183_2, epsilon = 0.001);
    }
}
