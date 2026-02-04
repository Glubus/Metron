pub mod difficulty;
pub mod performance;

use crate::calculator::{Calculator, CalculatorResult};
use rhythm_open_exchange::RoxChart;

pub struct Osu2015;

impl Calculator for Osu2015 {
    type Difficulty = difficulty::Osu2015Difficulty;
    type Performance = performance::Osu2015Performance;

    const NAME: &'static str = "osu!mania 2015";
    const VERSION: &'static str = "2015.1";
    const GAME: &'static str = "osu!mania";
    const YEAR: u32 = 2015;

    fn calculate_difficulty(&self, chart: &RoxChart) -> CalculatorResult<Self::Difficulty> {
        difficulty::calculate(chart)
    }

    fn calculate_performance(&self, difficulty: &Self::Difficulty, accuracy: f32) -> CalculatorResult<Self::Performance> {
        performance::calculate(difficulty, accuracy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osu2015_name() {
        let calc = Osu2015;
        assert_eq!(calc.name(), "osu!mania 2015");
    }

    #[test]
    fn test_osu2015_version() {
        let calc = Osu2015;
        assert_eq!(calc.version(), "2015.1");
    }

    #[test]
    fn test_osu2015_game() {
        let calc = Osu2015;
        assert_eq!(calc.game(), "osu!mania");
    }

    #[test]
    fn test_osu2015_calculate_difficulty() {
        let calc = Osu2015;
        let chart = RoxChart::new(4);
        let result = calc.calculate_difficulty(&chart).expect("Difficulty calculation failed");
        assert!((result.stars - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_osu2015_calculate_performance() {
        let calc = Osu2015;
        let chart = RoxChart::new(4);
        let diff = calc.calculate_difficulty(&chart).unwrap();
        let result = calc.calculate_performance(&diff, 1.0).expect("Performance calculation failed");
        assert!((result.pp - 0.0).abs() < f32::EPSILON);
    }
}
