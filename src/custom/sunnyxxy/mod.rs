pub mod difficulty;

use crate::calculator::{Calculator, CalculatorResult, Rating};
use crate::clock_rate::ClockRate;
use rox::RoxChart;

pub use difficulty::SunnyxxyDifficulty;

#[derive(Debug, Default)]
pub struct SunnyxxyDifficultyContext {
    pub clock_rate: Option<ClockRate>,
    pub overall_difficulty: Option<f32>,
}

#[derive(Debug, Default)]
pub struct SunnyxxyPerformanceContext;

#[derive(Debug)]
pub struct SunnyxxyPerformance;

impl Rating for SunnyxxyPerformance {}

pub struct SunnyXXY;

impl Calculator for SunnyXXY {
    type DifficultyContext = SunnyxxyDifficultyContext;
    type PerformanceContext = SunnyxxyPerformanceContext;

    type Difficulty = SunnyxxyDifficulty;
    type Performance = SunnyxxyPerformance;

    const NAME: &'static str = "Sunnyxxy Star Rating";
    const VERSION: &'static str = "0.2.1";
    const GAME: &'static str = "osu!mania";
    const YEAR: u32 = 2024;

    fn calculate_difficulty(
        &self,
        chart: &RoxChart,
        context: &Self::DifficultyContext,
    ) -> CalculatorResult<Self::Difficulty> {
        difficulty::calculate(chart, context)
    }
}
