use crate::quaver::quaver2025::difficulty::constants::StrainConstants;
use crate::quaver::quaver2025::difficulty::structs::{FingerAction, StrainSolverData};

/// Determines the finger action for each note (Roll, Jack, Bracket, etc.) and calculates the strain coefficient.
pub fn process_finger_actions(
    strain_solver_data: &mut Vec<StrainSolverData>,
    constants: &StrainConstants,
    average_note_density: f64,
) {
    // We need to iterate mutably but look ahead.
    // Index-based iteration is safest here.
    let len = strain_solver_data.len();
    if len < 2 {
        return;
    }

    for i in 0..len - 1 {
        // Find the next Hit Object in the current Hit Object's Hand
        for j in i + 1..len {
            // We need to check hand and time without borrowing conflicts.
            // Since we're iterating i, we can access strain_solver_data[i] and strain_solver_data[j].
            // But to set next_strain_solver_data_on_current_hand we need ownership or clone.
            // The original C# code sets a reference/pointer. In Rust we Box<Clone>.

            let hand_i = strain_solver_data[i].hand;
            let hand_j = strain_solver_data[j].hand;
            let start_time_i = strain_solver_data[i].start_time;
            let start_time_j = strain_solver_data[j].start_time;

            if hand_i == hand_j && start_time_j > start_time_i {
                // Determine finger action
                let finger_state_i = strain_solver_data[i].finger_state;
                let finger_state_j = strain_solver_data[j].finger_state;

                let action_jack_found = (finger_state_i.0 & finger_state_j.0) != 0; // Bitwise AND
                let action_chord_found =
                    strain_solver_data[i].hand_chord() || strain_solver_data[j].hand_chord();
                let action_same_state = finger_state_i == finger_state_j;
                let action_duration = start_time_j - start_time_i;

                // Set index to J
                strain_solver_data[i].next_strain_solver_index_on_current_hand = Some(j);
                strain_solver_data[i].finger_action_duration_ms = action_duration;

                // Determine action type and coefficient
                if !action_chord_found && !action_same_state {
                    strain_solver_data[i].finger_action = FingerAction::Roll;
                    strain_solver_data[i].action_strain_coefficient = get_coefficient_value(
                        action_duration,
                        constants.roll_lower_boundary_ms,
                        constants.roll_upper_boundary_ms,
                        constants.roll_max_strain_value,
                        constants.roll_curve_exponential,
                        average_note_density,
                    );
                } else if action_same_state {
                    strain_solver_data[i].finger_action = FingerAction::SimpleJack;
                    strain_solver_data[i].action_strain_coefficient = get_coefficient_value(
                        action_duration,
                        constants.s_jack_lower_boundary_ms,
                        constants.s_jack_upper_boundary_ms,
                        constants.s_jack_max_strain_value,
                        constants.s_jack_curve_exponential,
                        average_note_density,
                    );
                } else if action_jack_found {
                    strain_solver_data[i].finger_action = FingerAction::TechnicalJack;
                    strain_solver_data[i].action_strain_coefficient = get_coefficient_value(
                        action_duration,
                        constants.t_jack_lower_boundary_ms,
                        constants.t_jack_upper_boundary_ms,
                        constants.t_jack_max_strain_value,
                        constants.t_jack_curve_exponential,
                        average_note_density,
                    );
                } else {
                    strain_solver_data[i].finger_action = FingerAction::Bracket;
                    strain_solver_data[i].action_strain_coefficient = get_coefficient_value(
                        action_duration,
                        constants.bracket_lower_boundary_ms,
                        constants.bracket_upper_boundary_ms,
                        constants.bracket_max_strain_value,
                        constants.bracket_curve_exponential,
                        average_note_density,
                    );
                }
                break;
            }
        }
    }
}

/// Helper to calculate coefficient value based on duration and constants.
fn get_coefficient_value(
    duration: f64,
    x_min: f64,
    x_max: f64,
    strain_max: f64,
    exp: f64,
    average_note_density: f64,
) -> f64 {
    const LOWEST_DIFFICULTY: f64 = 1.0;
    const DENSITY_MULTIPLIER: f64 = 0.266;
    const DENSITY_DIFFICULTY_MIN: f64 = 0.4;

    // Calculate ratio between min and max value
    let ratio = (1.0 - (duration - x_min) / (x_max - x_min)).max(0.0);

    // If ratio is too big and map isn't a beginner map (nps > 4) scale based on nps instead
    if ratio == 0.0 && average_note_density < 4.0 {
        // If note density is too low don't bother calculating for density either
        if average_note_density < 1.0 {
            return DENSITY_DIFFICULTY_MIN;
        }
        return average_note_density * DENSITY_MULTIPLIER + 0.134;
    }

    // Compute for difficulty
    LOWEST_DIFFICULTY + (strain_max - LOWEST_DIFFICULTY) * ratio.powf(exp)
}
