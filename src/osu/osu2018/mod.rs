pub mod difficulty;
pub mod performance;

use crate::{
    calculator::{Calculator, CalculatorResult},
    clock_rate::ClockRate,
};
use rhythm_open_exchange::RoxChart;

#[derive(Debug, Default)]
pub struct Osu2018DifficultyContext {
    pub clock_rate: Option<ClockRate>,
    pub overall_difficulty: Option<f32>,
}

#[derive(Debug, Default)]
pub struct Osu2018PerformanceContext {
    pub accuracy: f32,
}

pub struct Osu2018;

impl Calculator for Osu2018 {
    type DifficultyContext = Osu2018DifficultyContext;
    type PerformanceContext = Osu2018PerformanceContext;

    type Difficulty = difficulty::Osu2018Difficulty;
    type Performance = performance::Osu2018Performance;

    const NAME: &'static str = "osu!mania 2018";
    const VERSION: &'static str = "2018.1";
    const GAME: &'static str = "osu!mania";
    const YEAR: u32 = 2018;

    fn calculate_difficulty(
        &self,
        chart: &RoxChart,
        context: &Self::DifficultyContext,
    ) -> CalculatorResult<Self::Difficulty> {
        difficulty::calculate(chart, context)
    }

    fn calculate_performance(
        &self,
        chart: &RoxChart,
        difficulty: &Self::Difficulty,
        context: &Self::PerformanceContext,
    ) -> CalculatorResult<Self::Performance> {
        performance::calculate(chart, difficulty, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osu2018_name() {
        let calc = Osu2018;
        assert_eq!(calc.name(), "osu!mania 2018");
    }

    #[test]
    fn test_osu2018_version() {
        let calc = Osu2018;
        assert_eq!(calc.version(), "2018.1");
    }

    #[test]
    fn test_osu2018_game() {
        let calc = Osu2018;
        assert_eq!(calc.game(), "osu!mania");
    }
}
