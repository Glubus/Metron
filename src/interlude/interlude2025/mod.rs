pub mod constants;
pub mod difficulty;
pub mod performance;
pub mod util;

use crate::calculator::{Calculator, CalculatorResult};
use rhythm_open_exchange::RoxChart;

pub use difficulty::Interlude2025Difficulty;
pub use performance::Interlude2025Performance;

#[derive(Debug, Default)]
pub struct Interlude2025DifficultyContext {
    pub clock_rate: Option<u32>,
}

#[derive(Debug, Default)]
pub struct Interlude2025PerformanceContext {
    pub accuracy: f32,
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
        let rate = context.clock_rate.unwrap_or(100) as f32 / 100.0;
        let stars = difficulty::calculate(chart, rate);
        Ok(Interlude2025Difficulty { stars })
    }

    fn calculate_performance(
        &self,
        _chart: &RoxChart,
        _difficulty: &Self::Difficulty,
        _context: &Self::PerformanceContext,
    ) -> CalculatorResult<Self::Performance> {
        Ok(Interlude2025Performance { pp: 0.0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhythm_open_exchange::auto_decode;

    #[test]
    fn test_calculate_difficulty_integration() {
        let chart = auto_decode("assets/test.osu").expect("Failed to decode test.osu");
        let calc = Interlude2025;
        let context = Interlude2025DifficultyContext { clock_rate: Some(100) };
        
        // We do not have a reference value yet, but we want to ensure it runs without panicking
        // and returns a finite positive value.
        let result = calc.calculate_difficulty(&chart, &context).expect("Calculation failed");
        
        println!("Calculated Stars: {}", result.stars);
        assert!(result.stars >= 0.0);
        assert!(result.stars.is_finite());
        
        // Optional: If we had a known value, we would assert it here.
        // For now, this proves integration with RoxChart works.
    }
}
