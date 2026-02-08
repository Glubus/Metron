pub mod performance;

use crate::calculator::{Calculator, CalculatorResult};
use crate::osu::osu2018::{Osu2018DifficultyContext, difficulty, difficulty::Osu2018Difficulty};
use rhythm_open_exchange::RoxChart;

#[derive(Debug, Default)]
pub struct Osu2016PerformanceContext {
    pub accuracy: f64,
    pub score: f64,
}

pub struct Osu2016;

impl Calculator for Osu2016 {
    type DifficultyContext = Osu2018DifficultyContext;
    type PerformanceContext = Osu2016PerformanceContext;

    type Difficulty = Osu2018Difficulty;
    type Performance = performance::Osu2016Performance;

    const NAME: &'static str = "osu!mania 2016";
    const VERSION: &'static str = "2016.1";
    const GAME: &'static str = "osu!mania";
    const YEAR: u32 = 2016;

    /// Reuses osu! 2018 Difficulty Calculation
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
