use crate::calculator::Rating;
use super::difficulty::Osu2015Difficulty;

#[derive(Debug)]
pub struct Osu2015Performance {
    pub pp: f32,
}

impl Rating for Osu2015Performance {}

pub fn calculate(_diff: &Osu2015Difficulty, _acc: f32) -> crate::calculator::CalculatorResult<Osu2015Performance> {
    // TDD stub
    Ok(Osu2015Performance { pp: 0.0 })
}
