use self::constants::StrainConstants;
use self::logic::{difficulty, fingering, ln, manipulation};
use self::structs::{FingerState, Hand, QuaverDifficulty, StrainSolverData, StrainSolverHitObject};
use crate::calculator::{Calculator, CalculatorResult};
use crate::clock_rate::ClockRate;
use rhythm_open_exchange::RoxChart;

pub mod constants;
pub mod logic;
pub mod structs;

/// Context for difficulty calculation
#[derive(Debug, Clone, Default)]
pub struct QuaverDifficultyContext {
    pub clock_rate: ClockRate,
}

/// Context for performance calculation (placeholder)
#[derive(Debug, Clone, Default)]
pub struct QuaverPerformanceContext;

/// Quaver 2025 Difficulty Calculator
pub struct Quaver2025;

impl Calculator for Quaver2025 {
    type DifficultyContext = QuaverDifficultyContext;
    type PerformanceContext = QuaverPerformanceContext;

    type Difficulty = QuaverDifficulty;
    type Performance = QuaverDifficulty; // Placeholder

    const NAME: &'static str = "Quaver";
    const VERSION: &'static str = "2025.1";
    const GAME: &'static str = "quaver";
    const YEAR: u32 = 2025;

    fn calculate_difficulty(
        &self,
        chart: &RoxChart,
        context: &Self::DifficultyContext,
    ) -> CalculatorResult<Self::Difficulty> {
        let key_count = chart.key_count();
        // Quaver logic is mainly for 4K and 7K but valid for 1-10K
        if key_count == 0 {
            return Ok(QuaverDifficulty { stars: 0.0 });
        }

        let constants = StrainConstants::default();
        let clock_rate: f64 = context.clock_rate.into();

        // 1. Initialize Strain Solver Data (Clustering is now done here)
        let mut strain_solver_data = initialize_strain_data(chart, key_count as i32, clock_rate, None, &constants);

        if strain_solver_data.is_empty() {
            return Ok(QuaverDifficulty { stars: 0.0 });
        }

        // 2. Average Note Density (needed for coefficients)
        // Original logic: SECONDS_TO_MILLISECONDS * count / (length * (-0.5 * clock_rate + 1.5))
        // Length of map in ms.
        let map_len = if chart.notes.is_empty() {
            1.0
        } else {
            let first = chart.notes.first().unwrap().time_us as f64 / 1000.0;
            let last = chart.notes.last().unwrap().duration_us().max(0) as f64 / 1000.0
                + chart.notes.last().unwrap().time_us as f64 / 1000.0;
            (last - first).max(1.0)
        };

        let average_note_density =
            1000.0 * chart.notes.len() as f64 / (map_len * (-0.5 * clock_rate + 1.5));

        // 3. Process Clustering (Merged into step 1)

        // 4. Process Finger Actions
        fingering::process_finger_actions(
            &mut strain_solver_data,
            &constants,
            average_note_density,
        );

        // 5. Process Manipulation (Rolls/Jacks)
        let mut vibro_confidence = 0.0;
        let mut roll_confidence = 0.0;
        manipulation::process_manipulation(
            &mut strain_solver_data,
            &constants,
            &mut vibro_confidence,
            &mut roll_confidence,
        );

        // 6. Process LN Layers
        ln::process_ln_layers(&mut strain_solver_data, &constants);

        // 7. Calculate Final Difficulty
        let use_fallback = key_count % 2 == 1; // Odd Key Count Fallback

        // Calculate overall difficulty per hand if even key count
        let stars = if key_count % 2 == 0 {
            difficulty::calculate_final_difficulty(&mut strain_solver_data, use_fallback)
        } else {
            // First run (Left bias)
            let mut data_left = initialize_strain_data(chart, key_count as i32, clock_rate, Some(Hand::Left), &constants);
            fingering::process_finger_actions(&mut data_left, &constants, average_note_density);
            manipulation::process_manipulation(&mut data_left, &constants, &mut 0.0, &mut 0.0);
            ln::process_ln_layers(&mut data_left, &constants);
            let diff_left = difficulty::calculate_final_difficulty(&mut data_left, use_fallback);

            // Second run (Right bias)
            let mut data_right = initialize_strain_data(chart, key_count as i32, clock_rate, Some(Hand::Right), &constants);
            fingering::process_finger_actions(&mut data_right, &constants, average_note_density);
            manipulation::process_manipulation(&mut data_right, &constants, &mut 0.0, &mut 0.0);
            ln::process_ln_layers(&mut data_right, &constants);
            let diff_right = difficulty::calculate_final_difficulty(&mut data_right, use_fallback);

            (diff_left + diff_right) / 2.0
        };

        Ok(QuaverDifficulty { stars })
    }

}

// Helpers

fn initialize_strain_data(
    chart: &RoxChart,
    key_count: i32,
    clock_rate: f64,
    override_ambiguous_hand: Option<Hand>,
    constants: &StrainConstants,
) -> Vec<StrainSolverData> {
    let mut data: Vec<StrainSolverData> = Vec::with_capacity(chart.notes.len());
    let mut last_left_idx: Option<usize> = None;
    let mut last_right_idx: Option<usize> = None;
    let mut last_ambiguous_idx: Option<usize> = None;

    // Optimization: avoid clone/sort if possible, but for safety in this refactor,
    // we stick to sorting to ensure time order (vital for clustering).
    // If we trust RoxChart to be sorted, we could skip this.
    // For now, let's keep it robust.
    let mut notes = chart.notes.clone();
    notes.sort_by_key(|n| n.time_us);

    for note in notes {
        let lane = note.column as i32 + 1;
        let start_time = note.time_us as f64 / 1000.0 / clock_rate;
        let end_time = (note.time_us + note.duration_us()) as f64 / 1000.0 / clock_rate;

        let mut hit_obj_check = StrainSolverHitObject::new(start_time, lane);
        hit_obj_check.end_time = end_time; // We just use this for checking fields initially

        // We need the object itself.
        // Let's use Option to handle move
        let mut hit_obj_opt = Some(hit_obj_check);
        if let Some(finger) = lane_to_finger(lane, key_count) {
             hit_obj_opt.as_mut().unwrap().finger_state = finger;
        }
        // hit_obj_opt is populated.

        // Determine hand
        let mut hand = Hand::Ambiguous;
        if let Some(h) = lane_to_hand(lane, key_count) {
            hand = h;
        }
        if hand == Hand::Ambiguous {
            if let Some(override_hand) = override_ambiguous_hand {
                hand = override_hand;
            }
        }

        // Determine which cluster key to check
        let check_idx = match hand {
            Hand::Left => last_left_idx,
            Hand::Right => last_right_idx,
            Hand::Ambiguous => last_ambiguous_idx,
        };

        // Attempt to cluster with the last element of the SAME HAND
        let mut merged = false;

        if let Some(idx) = check_idx {
            // We must use get_mut separately to avoid borrow checker issues?
            // data is a Vec. We can index it.
            if let Some(last) = data.get_mut(idx) {
                let hit_obj_ref = hit_obj_opt.as_ref().unwrap();
                let ms_diff = hit_obj_ref.start_time - last.start_time;

                // Note: last.hand == hand is guaranteed by our index tracking
                if ms_diff.abs() <= constants.chord_clump_tolerance_ms {
                    // Merge
                    let mut duplicate = false;
                    for existing in &last.hit_objects {
                        if existing.finger_state == hit_obj_ref.finger_state {
                            duplicate = true;
                            break;
                        }
                    }

                    if !duplicate {
                        last.hit_objects.push(hit_obj_opt.take().unwrap());
                    } else {
                        // Discard
                        hit_obj_opt = None;
                    }
                    merged = true;
                }
            }
        }

        if !merged {
            // If not merged, we must have the object (it wasn't moved or discarded)
            if let Some(hit_obj) = hit_obj_opt {
                 let mut strain_data = StrainSolverData::new(hit_obj);
                 strain_data.hand = hand;
                 data.push(strain_data);

                 // Update index
                 let new_idx = data.len() - 1;
                 match hand {
                    Hand::Left => last_left_idx = Some(new_idx),
                    Hand::Right => last_right_idx = Some(new_idx),
                    Hand::Ambiguous => last_ambiguous_idx = Some(new_idx),
                 }
            }
        }
    }

    // Solve finger states
    for d in &mut data {
        d.solve_finger_state();
    }

    data
}

// Logic from helpers.rs ported here

fn lane_to_hand(lane: i32, key_count: i32) -> Option<Hand> {
    if lane < 1 || lane > key_count {
        return None;
    }

    let half = key_count / 2;

    if key_count % 2 == 0 {
        if lane <= half {
            Some(Hand::Left)
        } else {
            Some(Hand::Right)
        }
    } else {
        if lane <= half {
            Some(Hand::Left)
        } else if lane == half + 1 {
            Some(Hand::Ambiguous)
        } else {
            Some(Hand::Right)
        }
    }
}

fn lane_to_finger(lane: i32, key_count: i32) -> Option<FingerState> {
    if lane < 1 || lane > key_count {
        return None;
    }

    let half = key_count / 2;

    if key_count <= 9 {
        // even key count
        if key_count % 2 == 0 {
            if lane <= half {
                FingerState::from_bits(1 << (half - lane))
            } else {
                FingerState::from_bits(1 << (lane - (half + 1)))
            }
        } else {
            // odd key count
            if lane <= half {
                FingerState::from_bits(1 << (half - lane))
            } else if lane == half + 1 {
                Some(FingerState::THUMB)
            } else {
                FingerState::from_bits(1 << (lane - (half + 2)))
            }
        }
    } else if key_count == 10 {
        if lane <= half - 1 {
            FingerState::from_bits(1 << (half - 1 - lane))
        } else if lane == half || lane == half + 1 {
            Some(FingerState::THUMB)
        } else {
            FingerState::from_bits(1 << (lane - (half + 2)))
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use rhythm_open_exchange::auto_decode;
    use rstest::*;

    #[fixture]
    fn chart() -> RoxChart {
        // We assume the test is running from the project root
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/test.osu");
        auto_decode(path).expect("Failed to decode test.osu")
    }

    #[rstest]
    fn test_quaver_difficulty_calculation(chart: RoxChart) {
        let calc = Quaver2025;
        let context = QuaverDifficultyContext {
            clock_rate: ClockRate::default(),
        };

        let result = calc.calculate_difficulty(&chart, &context);

        assert!(result.is_ok());
        let difficulty = result.unwrap();
        println!("Quaver Difficulty: {:?}", difficulty);
        assert_abs_diff_eq!(difficulty.stars, 29.81354, epsilon = 0.001);
    }

    #[rstest]
    fn test_quaver_difficulty_calculation_rate(chart: RoxChart) {
        let calc = Quaver2025;
        let context = QuaverDifficultyContext {
            clock_rate: ClockRate::from_percentage(150).unwrap(),
        };

        let result = calc.calculate_difficulty(&chart, &context);

        assert!(result.is_ok());
        let difficulty = result.unwrap();
        println!("Quaver Difficulty (1.5x): {:?}", difficulty);
        assert_abs_diff_eq!(difficulty.stars, 41.28641, epsilon = 0.001);
    }
}
