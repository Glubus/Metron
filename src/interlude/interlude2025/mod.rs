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
    #[case(70,   6.005_243_833_356_738)]
    #[case(80,   7.025_572_662_735_565)]
    #[case(90,   8.075_650_522_598_215)]
    #[case(100,  9.120_204_235_501_205)]
    #[case(110, 10.106_061_098_546_48)]
    #[case(120, 11.090_076_508_639_688)]
    #[case(130, 12.127_759_324_966_58)]
    #[case(140, 13.205_845_600_614_897)]
    #[case(150, 14.095_243_032_324_886)]
    #[case(160, 14.701_744_162_660_29)]
    fn test_difficulty_at_rate(chart: RoxChart, #[case] rate_pct: u32, #[case] expected: f64) {
        let calc = Interlude2025;
        let context = Interlude2025DifficultyContext {
            clock_rate: Some(ClockRate::from_percentage(rate_pct).unwrap()),
        };
        let result = calc.calculate_difficulty(&chart, &context).unwrap();
        assert_abs_diff_eq!(result.stars, expected, epsilon = 1e-6);
    }
}
