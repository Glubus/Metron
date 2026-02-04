use crate::calculator::Rating;

#[derive(Debug)]
pub struct Osu2015Difficulty {
    pub stars: f32,
}

impl Rating for Osu2015Difficulty {}

pub fn calculate(_chart: &rhythm_open_exchange::RoxChart) -> crate::calculator::CalculatorResult<Osu2015Difficulty> {
    // TDD stub
    Ok(Osu2015Difficulty { stars: 0.0 })
}
