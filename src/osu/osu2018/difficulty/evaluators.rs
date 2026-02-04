use rhythm_open_exchange::RoxChart;
use std::f32::consts::E;

pub struct Evaluator;

impl Evaluator {
    // Individual Strain
    pub fn evaluate_individual(
        chart: &RoxChart,
        _current_idx: usize,
        current_time: f32,
        current_end_time: f32,
        history: &[Option<usize>],
        clock_rate: f32
    ) -> f32 {
        let mut hold_factor = 1.0;

        for &prev_idx_opt in history {
            if let Some(prev_idx) = prev_idx_opt {
                let prev_note = &chart.notes[prev_idx];
                let prev_start = prev_note.time_us as f32 / 1_000.0 / clock_rate;
                let prev_end = prev_start + (prev_note.duration_us() as f32 / 1_000.0 / clock_rate);
                
                // Precision check: definitely bigger ~ > 1ms margin
                // prev.EndTime > current.EndTime + 1
                // current.StartTime > prev.StartTime + 1
                if (prev_end > current_end_time + 1.0) && (current_time > prev_start + 1.0) {
                    hold_factor = 1.25;
                    break;
                }
            }
        }

        2.0 * hold_factor
    }

    // Overall Strain
    pub fn evaluate_overall(
        chart: &RoxChart,
        _current_idx: usize,
        current_time: f32,
        current_end_time: f32,
        history: &[Option<usize>],
        clock_rate: f32
    ) -> f32 {
        let mut closest_end_time = (current_end_time - current_time).abs();
        let mut hold_factor = 1.0;
        let mut hold_addition = 0.0;
        let mut is_overlapping = false;

        for &prev_idx_opt in history {
            if let Some(prev_idx) = prev_idx_opt {
                let prev_note = &chart.notes[prev_idx];
                let prev_start = prev_note.time_us as f32 / 1_000.0 / clock_rate;
                let prev_end = prev_start + (prev_note.duration_us() as f32 / 1_000.0 / clock_rate);
                
                // Overlap: prev.EndTime > Start + 1 && End > prev.EndTime + 1 && Start > prev.Start + 1
                let overlap = (prev_end > current_time + 1.0) && 
                              (current_end_time > prev_end + 1.0) && 
                              (current_time > prev_start + 1.0);
                              
                if overlap {
                    is_overlapping = true;
                }

                if (prev_end > current_end_time + 1.0) && (current_time > prev_start + 1.0) {
                    hold_factor = 1.25;
                }

                closest_end_time = closest_end_time.min((current_end_time - prev_end).abs());
            }
        }

        if is_overlapping {
            hold_addition = logistic(closest_end_time, 0.27, 30.0);
        }

        (1.0 + hold_addition) * hold_factor
    }
}

fn logistic(x: f32, multiplier: f32, midpoint_offset: f32) -> f32 {
    1.0 / (1.0 + E.powf(-multiplier * (x - midpoint_offset)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhythm_open_exchange::{auto_decode, RoxChart};
    use rstest::*;
    use approx::assert_abs_diff_eq;

    #[fixture]
    fn chart() -> RoxChart {
        auto_decode("assets/test.osu").expect("Failed to decode test.osu")
    }

    #[rstest]
    fn test_logistic_curve() {
        // midpoint
        assert_abs_diff_eq!(logistic(30.0, 0.27, 30.0), 0.5, epsilon = 0.001);
        // far left
        assert!(logistic(0.0, 0.27, 30.0) < 0.01);
        // far right
        assert!(logistic(60.0, 0.27, 30.0) > 0.99);
    }

    #[rstest]
    fn test_evaluate_individual_no_history(chart: RoxChart) {
        let current_time = 1000.0;
        let current_end_time = 1100.0;
        let history = vec![None, None, None, None];
        let clock_rate = 1.0;

        let strain = Evaluator::evaluate_individual(
            &chart,
            0,
            current_time,
            current_end_time,
            &history,
            clock_rate
        );

        // Base hold factor is 1.0, result is 2.0 * hold_factor
        assert_abs_diff_eq!(strain, 2.0, epsilon = 0.001);
    }

    #[rstest]
    fn test_evaluate_overall_no_overlap(chart: RoxChart) {
        let current_time = 1000.0;
        let current_end_time = 1100.0;
        let history = vec![None];
        let clock_rate = 1.0;

        let strain = Evaluator::evaluate_overall(
            &chart,
            0,
            current_time,
            current_end_time,
            &history,
            clock_rate
        );

        // No overlap, no hold addition -> (1.0 + 0.0) * 1.0 = 1.0
        assert_abs_diff_eq!(strain, 1.0, epsilon = 0.001);
    }
}
