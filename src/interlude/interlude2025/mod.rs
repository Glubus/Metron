pub mod constants;
pub mod difficulty;
pub mod performance;
pub mod util;

use crate::calculator::{Calculator, CalculatorResult};
use crate::clock_rate::ClockRate;
use rhythm_open_exchange::RoxChart;

pub use difficulty::Interlude2025Difficulty;
pub use performance::Interlude2025Performance;

#[derive(Debug, Default)]
pub struct Interlude2025DifficultyContext {
    pub clock_rate: Option<ClockRate>,
}

#[derive(Debug, Default)]
pub struct Interlude2025PerformanceContext {
    pub replay: i32, // TODO type replay
}

pub struct Interlude2025;

impl Calculator for Interlude2025 {
    type DifficultyContext = Interlude2025DifficultyContext;
    type PerformanceContext = Interlude2025PerformanceContext;

    type Difficulty = Interlude2025Difficulty;
    type Performance = Interlude2025Performance;

    const NAME: &'static str = "Interlude 2025";
    const VERSION: &'static str = "2025.1";
    const GAME: &'static str = "osu!mania";
    const YEAR: u32 = 2025;

    fn calculate_difficulty(
        &self,
        chart: &RoxChart,
        context: &Self::DifficultyContext,
    ) -> CalculatorResult<Self::Difficulty> {
        let stars = difficulty::calculate(chart, context);
        Ok(Interlude2025Difficulty { stars })
    }

    fn calculate_performance(
        &self,
        _chart: &RoxChart,
        _difficulty: &Self::Difficulty,
        _context: &Self::PerformanceContext,
    ) -> CalculatorResult<Self::Performance> {
        Ok(Interlude2025Performance { ratings: 0.0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use rhythm_open_exchange::auto_decode;
    use rstest::{fixture, rstest};

    #[fixture]
    fn chart() -> RoxChart {
        auto_decode("assets/test.osu").expect("Failed to decode test.osu")
    }

    #[rstest]
    fn test_calculate_difficulty_integration(chart: RoxChart) {
        let calc = Interlude2025;
        let context = Interlude2025DifficultyContext {
            clock_rate: Some(ClockRate::from_percentage(100).expect("Valid clock rate")),
        };

        // We do not have a reference value yet, but we want to ensure it runs without panicking
        // and returns a finite positive value.
        let result = calc
            .calculate_difficulty(&chart, &context)
            .expect("Calculation failed");

        println!("Calculated Stars: {}", result.stars);
        assert!(result.stars >= 0.0);
        assert!(result.stars.is_finite());
        assert_abs_diff_eq!(result.stars, 9.120_204_235_501_202, epsilon = 0.001);
    }
}
