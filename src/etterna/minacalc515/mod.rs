use crate::calculator::{Calculator, CalculatorError, CalculatorResult, Rating};
use crate::clock_rate::ClockRate;
use crate::etterna::convert::chart_to_notes;
use minacalc_rs::{Calc, CalcMode};
use rox::RoxChart;
use std::cell::RefCell;

// One Calc per thread — Calc is not Send (underlying C++ is not thread-safe).
thread_local! {
    static CALC: RefCell<Calc> = RefCell::new(
        Calc::new().expect("Failed to allocate MinaCalc instance")
    );
}

fn with_calc<F, T>(f: F) -> Result<T, CalculatorError>
where
    F: FnOnce(&Calc) -> Result<T, minacalc_rs::Error>,
{
    CALC.with(|c| f(&c.borrow()).map_err(|e| CalculatorError::Calculation(e.to_string())))
}

#[derive(Debug, Clone, Copy)]
pub struct MinaCalcDifficultyContext {
    pub clock_rate: ClockRate,
    /// `CalcMode::Msd` = normal/uncapped (default, used on etternaonline.com).
    /// `CalcMode::Ssr` = nerfed/capped — some integrations use this silently.
    pub mode: CalcMode,
}

impl Default for MinaCalcDifficultyContext {
    fn default() -> Self {
        Self {
            clock_rate: ClockRate::default(),
            mode: CalcMode::Msd,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EtternaDifficulty {
    pub overall: f32,
    pub stream: f32,
    pub jumpstream: f32,
    pub handstream: f32,
    pub stamina: f32,
    pub jackspeed: f32,
    pub chordjack: f32,
    pub technical: f32,
}

impl Rating for EtternaDifficulty {}

/// Context for performance calculation.
///
/// Defaults to `CalcMode::Ssr` (nerfed, capped at player accuracy) since that
/// is the typical use-case for a score-relative rating.
/// Switch to `CalcMode::Msd` if you want the uncapped version at a given accuracy.
#[derive(Debug, Clone, Copy)]
pub struct MinaCalcPerformanceContext {
    pub clock_rate: ClockRate,
    /// Player's accuracy / score goal (e.g. 0.93 for 93%).
    pub score_goal: f32,
    /// `CalcMode::Ssr` = nerfed/capped (default). `CalcMode::Msd` = uncapped.
    pub mode: CalcMode,
}

impl Default for MinaCalcPerformanceContext {
    fn default() -> Self {
        Self {
            clock_rate: ClockRate::default(),
            score_goal: 0.93,
            mode: CalcMode::Ssr,
        }
    }
}

pub struct MinaCalc515;

impl Calculator for MinaCalc515 {
    type DifficultyContext = MinaCalcDifficultyContext;
    type PerformanceContext = MinaCalcPerformanceContext;

    type Difficulty = EtternaDifficulty;
    type Performance = EtternaDifficulty;

    const NAME: &'static str = "MinaCalc";
    const VERSION: &'static str = "515";
    const GAME: &'static str = "etterna";
    const YEAR: u32 = 2021;

    /// Raw chart difficulty (MSD — normal mode, as shown on etternaonline.com).
    fn calculate_difficulty(
        &self,
        chart: &RoxChart,
        context: &Self::DifficultyContext,
    ) -> CalculatorResult<Self::Difficulty> {
        let key_count = chart.key_count as u32;
        if key_count != 4 && key_count != 6 && key_count != 7 {
            return Err(CalculatorError::Calculation(format!(
                "Unsupported key count: {key_count}"
            )));
        }

        let clock_rate: f64 = context.clock_rate.into();
        let notes = chart_to_notes(chart, clock_rate)?;

        let scores = with_calc(|calc| {
            calc.calc_at_rate(&notes, clock_rate as f32, 0.93, key_count, context.mode)
        })?;

        Ok(EtternaDifficulty {
            overall: scores.overall,
            stream: scores.stream,
            jumpstream: scores.jumpstream,
            handstream: scores.handstream,
            stamina: scores.stamina,
            jackspeed: scores.jackspeed,
            chordjack: scores.chordjack,
            technical: scores.technical,
        })
    }

    /// Score-relative rating (SSR — nerfed, capped at player accuracy).
    fn calculate_performance(
        &self,
        chart: &RoxChart,
        _difficulty: &Self::Difficulty,
        context: &Self::PerformanceContext,
    ) -> CalculatorResult<Self::Performance> {
        let key_count = chart.key_count as u32;
        if key_count != 4 && key_count != 6 && key_count != 7 {
            return Err(CalculatorError::Calculation(format!(
                "Unsupported key count: {key_count}"
            )));
        }

        let clock_rate: f64 = context.clock_rate.into();
        let notes = chart_to_notes(chart, clock_rate)?;

        let scores = with_calc(|calc| {
            calc.calc_at_rate(&notes, clock_rate as f32, context.score_goal, key_count, context.mode)
        })?;

        Ok(EtternaDifficulty {
            overall: scores.overall,
            stream: scores.stream,
            jumpstream: scores.jumpstream,
            handstream: scores.handstream,
            stamina: scores.stamina,
            jackspeed: scores.jackspeed,
            chordjack: scores.chordjack,
            technical: scores.technical,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rox_formats::auto_decode;

    #[test]
    fn test_minacalc515_difficulty() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/test.osu");
        let chart = auto_decode(path).expect("Failed to decode test.osu");

        let calc = MinaCalc515;
        let context = MinaCalcDifficultyContext::default();

        let result = calc.calculate_difficulty(&chart, &context);
        assert!(result.is_ok());
        let diff = result.unwrap();
        println!("MinaCalc515 overall: {}", diff.overall);
        assert!(diff.overall > 0.0);
    }
}
