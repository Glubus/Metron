use crate::quaver::quaver2025::difficulty::constants::StrainConstants;
use crate::quaver::quaver2025::difficulty::structs::{FingerAction, StrainSolverData};

pub fn process_manipulation(
    strain_solver_data: &mut Vec<StrainSolverData>,
    constants: &StrainConstants,
    vibro_inaccuracy_confidence: &mut f64,
    roll_inaccuracy_confidence: &mut f64,
) {
    process_roll_manipulation(strain_solver_data, constants, roll_inaccuracy_confidence);
    process_jack_manipulation(strain_solver_data, constants, vibro_inaccuracy_confidence);
}

fn process_roll_manipulation(
    strain_solver_data: &mut Vec<StrainSolverData>,
    constants: &StrainConstants,
    roll_inaccuracy_confidence: &mut f64,
) {
    let mut manipulation_updates: Vec<(usize, f64)> = Vec::new(); // (index, multiplier)
    let mut manipulation_index = 0;

    for i in 0..strain_solver_data.len() {
        let mut manipulation_found = false;
        let data = &strain_solver_data[i];

        if let Some(n_idx) = data.next_strain_solver_index_on_current_hand {
            let next = &strain_solver_data[n_idx];

            if let Some(nn_idx) = next.next_strain_solver_index_on_current_hand {
                let next_next = &strain_solver_data[nn_idx];

                if data.finger_action == FingerAction::Roll
                    && next.finger_action == FingerAction::Roll
                {
                    if data.finger_state == next_next.finger_state {
                        let duration_ratio = (data.finger_action_duration_ms
                            / next.finger_action_duration_ms)
                            .max(next.finger_action_duration_ms / data.finger_action_duration_ms);

                        if duration_ratio >= constants.roll_ratio_tolerance_ms {
                            let duration_multiplier = 1.0
                                / (1.0 + (duration_ratio - 1.0) * constants.roll_ratio_multiplier);
                            let manipulation_found_ratio = 1.0
                                - manipulation_index as f64 / constants.roll_max_length
                                    * (1.0 - constants.roll_length_multiplier);

                            manipulation_updates
                                .push((i, duration_multiplier * manipulation_found_ratio));

                            manipulation_found = true;
                            *roll_inaccuracy_confidence += 1.0;

                            if manipulation_index < constants.roll_max_length as usize {
                                manipulation_index += 1;
                            }
                        }
                    }
                }
            }
        }

        if !manipulation_found && manipulation_index > 0 {
            manipulation_index -= 1;
        }
    }

    // Apply updates
    for (idx, multiplier) in manipulation_updates {
        strain_solver_data[idx].roll_manipulation_strain_multiplier = multiplier;
    }
}

fn process_jack_manipulation(
    strain_solver_data: &mut Vec<StrainSolverData>,
    constants: &StrainConstants,
    vibro_inaccuracy_confidence: &mut f64,
) {
    let mut manipulation_updates: Vec<(usize, f64)> = Vec::new(); // (index, multiplier)
    let mut long_jack_size = 0;

    for i in 0..strain_solver_data.len() {
        let mut manipulation_found = false;
        let data = &strain_solver_data[i];

        if let Some(n_idx) = data.next_strain_solver_index_on_current_hand {
            let next = &strain_solver_data[n_idx];

            if data.finger_action == FingerAction::SimpleJack
                && next.finger_action == FingerAction::SimpleJack
            {
                let duration_value = ((constants.vibro_action_duration_ms
                    + constants.vibro_action_tolerance_ms
                    - data.finger_action_duration_ms)
                    / constants.vibro_action_tolerance_ms)
                    .min(1.0)
                    .max(0.0);

                let duration_multiplier = 1.0 - duration_value * (1.0 - constants.vibro_multiplier);
                let manipulation_found_ratio = 1.0
                    - long_jack_size as f64 / constants.vibro_max_length
                        * (1.0 - constants.vibro_length_multiplier);

                manipulation_updates.push((i, duration_multiplier * manipulation_found_ratio));

                manipulation_found = true;
                *vibro_inaccuracy_confidence += 1.0;

                if long_jack_size < constants.vibro_max_length as usize {
                    long_jack_size += 1;
                }
            }
        }

        if !manipulation_found {
            long_jack_size = 0;
        }
    }

    // Apply updates
    for (idx, multiplier) in manipulation_updates {
        strain_solver_data[idx].roll_manipulation_strain_multiplier = multiplier;
    }
}
