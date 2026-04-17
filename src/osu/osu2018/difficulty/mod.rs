use super::Osu2018DifficultyContext;
use crate::calculator::Rating;
use rhythm_open_exchange::RoxChart;

pub mod evaluators;
pub mod strain;

use strain::Strain;

#[derive(Debug)]
pub struct Osu2018Difficulty {
    pub stars: f64,
    pub great_hit_window: f64,
    pub score_multiplier: f64,
    pub object_count: u32,
}

impl Rating for Osu2018Difficulty {}

/// Calculates the difficulty of a map.
///
/// # Errors
///
/// This function will return an error if the difficulty calculation fails.
#[allow(clippy::cast_precision_loss)]
pub fn calculate(
    chart: &RoxChart,
    context: &Osu2018DifficultyContext,
) -> crate::calculator::CalculatorResult<Osu2018Difficulty> {
    let clock_rate = f64::from(context.clock_rate.unwrap_or_default());

    // Extract OD/KeyCount
    let od_input = f64::from(context.overall_difficulty.unwrap_or(5.0));
    let column_count = chart.key_count as usize;

    // 1. Hit Window Calculation
    let safe_od: f64 = (10.0 - od_input).clamp(0.0, 10.0);
    let base_window = 34.0 + 3.0 * safe_od;
    let real_window = base_window / clock_rate;

    // 2. Strain Calculation
    let mut strain_skill = Strain::new(column_count);

    // Create sorted indices to iterate notes in time order without moving them
    let mut sorted_indices: Vec<usize> = (0..chart.notes.len()).collect();
    sorted_indices.sort_by(|&a, &b| chart.notes[a].time_us.cmp(&chart.notes[b].time_us));

    // Iteration state
    let mut last_object_per_column: Vec<Option<usize>> = vec![None; column_count];
    let mut last_note_time = 0.0; // For global delta_time

    for (i, &note_idx) in sorted_indices.iter().enumerate() {
        let note = &chart.notes[note_idx];
        let current_time = note.time_us as f64 / 1_000.0 / clock_rate; // ms
        let column = (note.column as usize).min(column_count - 1);

        // Calculate Delta Time (Global)
        let delta_time = if i > 0 {
            (current_time - last_note_time).max(0.0)
        } else {
            0.0
        };

        // Calculate Column Strain Time
        let column_strain_time = if let Some(prev_idx) = last_object_per_column[column] {
            let prev_time = chart.notes[prev_idx].time_us as f64 / 1_000.0 / clock_rate;
            current_time - prev_time
        } else {
            current_time // Or 0.0? Consistent with previous impl
        };

        // Process Strain
        strain_skill.process(
            chart,
            note_idx,
            current_time,
            delta_time,
            column_strain_time,
            &last_object_per_column,
            clock_rate,
        );

        // Update state
        last_object_per_column[column] = Some(note_idx);
        last_note_time = current_time;
    }

    let stars = strain_skill.difficulty_value() * 0.018;

    Ok(Osu2018Difficulty {
        stars,
        great_hit_window: real_window,
        score_multiplier: 1.0,
        object_count: u32::try_from(chart.notes.len()).map_err(|_| {
            crate::calculator::CalculatorError::Calculation("Too many notes".to_string())
        })?,
    })
}

#[cfg(test)]
mod tests {
    use crate::clock_rate::ClockRate;

    use super::*;
    use approx::assert_abs_diff_eq;
    use rox_formats::auto_decode;
    use rstest::{fixture, rstest};

    #[fixture]
    fn chart() -> RoxChart {
        auto_decode("assets/test.osu").expect("Failed to decode test.osu")
    }

    #[rstest]
    fn test_osu2018_difficulty_calculation(chart: RoxChart) {
        let context = Osu2018DifficultyContext {
            clock_rate: None,
            overall_difficulty: Some(8.0),
        };
        let result = calculate(&chart, &context).expect("Difficulty calculation failed");

        assert_abs_diff_eq!(result.stars, 6.537_083, epsilon = 0.001);
    }

    #[rstest]
    fn test_osu2018_difficulty_clock_rate(chart: RoxChart) {
        let context = Osu2018DifficultyContext {
            clock_rate: Some(ClockRate::from_percentage(200).expect("Valid clock rate")),
            overall_difficulty: Some(8.0),
        };
        let result = calculate(&chart, &context).expect("Difficulty calculation failed");

        assert_abs_diff_eq!(result.stars, 11.3172, epsilon = 0.001);
    }
}
