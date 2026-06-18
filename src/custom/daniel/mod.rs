pub mod difficulty;

use crate::calculator::{Calculator, CalculatorResult, Rating};
use crate::clock_rate::ClockRate;
use rhythm_open_exchange::RoxChart;

pub use difficulty::{
    DanielDifficulty, DanielDifficultyGraph, DanielFactorAverages, DanielFactorCurves,
};

#[derive(Debug, Default)]
pub struct DanielDifficultyContext {
    pub clock_rate: Option<ClockRate>,
    pub overall_difficulty: Option<f32>,
}

#[derive(Debug, Default)]
pub struct DanielPerformanceContext;

#[derive(Debug)]
pub struct DanielPerformance;

impl Rating for DanielPerformance {}

pub struct Daniel;

impl Calculator for Daniel {
    type DifficultyContext = DanielDifficultyContext;
    type PerformanceContext = DanielPerformanceContext;

    type Difficulty = DanielDifficulty;
    type Performance = DanielPerformance;

    const NAME: &'static str = "Daniel Star Rating";
    const VERSION: &'static str = "0.1.0";
    const GAME: &'static str = "osu!mania";
    const YEAR: u32 = 2026;

    fn calculate_difficulty(
        &self,
        chart: &RoxChart,
        context: &Self::DifficultyContext,
    ) -> CalculatorResult<Self::Difficulty> {
        difficulty::calculate(chart, context)
    }
}
