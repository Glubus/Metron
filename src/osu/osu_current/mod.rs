use crate::calculator::{Calculator, CalculatorError, CalculatorResult, Rating};
use crate::osu::osu2018::Osu2018DifficultyContext;
use rox_formats::osu::OsuEncoder;
use rhythm_open_exchange::{Encoder, RoxChart};
use std::str::FromStr;

#[derive(Debug)]
pub struct OsuCurrentDifficulty {
    pub stars: f64,
}

impl Rating for OsuCurrentDifficulty {}

#[derive(Debug, Default)]
pub struct OsuCurrentPerformanceContext;

pub struct OsuCurrent;

impl Calculator for OsuCurrent {
    type DifficultyContext = Osu2018DifficultyContext;
    type PerformanceContext = OsuCurrentPerformanceContext;
    type Difficulty = OsuCurrentDifficulty;
    type Performance = OsuCurrentDifficulty;

    const NAME: &'static str = "osu!mania current";
    const VERSION: &'static str = "lazer";
    const GAME: &'static str = "osu!mania";
    const YEAR: u32 = 2025;

    fn calculate_difficulty(
        &self,
        chart: &RoxChart,
        context: &Self::DifficultyContext,
    ) -> CalculatorResult<Self::Difficulty> {
        let clock_rate = f64::from(context.clock_rate.unwrap_or_default());
        let osu_bytes = OsuEncoder::encode(chart)
            .map_err(|e| CalculatorError::Calculation(e.to_string()))?;
        let osu_str = String::from_utf8(osu_bytes)
            .map_err(|e| CalculatorError::Calculation(e.to_string()))?;
        let map = rosu_pp::Beatmap::from_str(&osu_str)
            .map_err(|e| CalculatorError::Calculation(e.to_string()))?;
        let stars = rosu_pp::Difficulty::new()
            .clock_rate(clock_rate)
            .calculate(&map)
            .stars();
        Ok(OsuCurrentDifficulty { stars })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rox_formats::auto_decode;

    #[test]
    fn test_osu_current_difficulty() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/test.osu");
        let chart = auto_decode(path).expect("Failed to decode test.osu");
        let calc = OsuCurrent;
        let ctx = Osu2018DifficultyContext { clock_rate: None, overall_difficulty: None };
        let result = calc.calculate_difficulty(&chart, &ctx);
        assert!(result.is_ok(), "Error: {:?}", result.err());
        assert!(result.unwrap().stars > 0.0);
    }
}
