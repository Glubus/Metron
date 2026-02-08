use crate::quaver::quaver2025::difficulty::constants::StrainConstants;
use crate::quaver::quaver2025::difficulty::structs::{LnLayerType, StrainSolverData};

pub fn process_ln_layers(
    strain_solver_data: &mut Vec<StrainSolverData>,
    constants: &StrainConstants,
) {
    let mut updates: Vec<(usize, usize, f64, LnLayerType)> = Vec::new(); // (data_idx, hit_obj_idx, multiplier, layer_type)

    for (i, data) in strain_solver_data.iter().enumerate() {
        if data.end_time > data.start_time {
            let duration_value = 1.0
                - ((constants.ln_layer_threshold_ms + constants.ln_layer_tolerance_ms
                    - (data.end_time - data.start_time))
                    / constants.ln_layer_tolerance_ms)
                    .min(1.0)
                    .max(0.0);

            let base_multiplier = 1.0 + duration_value * constants.ln_base_multiplier;

            let mut layer_type_update = LnLayerType::None;
            let mut multiplier_factor = 1.0;

            if let Some(n_idx) = data.next_strain_solver_index_on_current_hand {
                let next = &strain_solver_data[n_idx];

                if next.start_time < data.end_time - constants.ln_end_threshold_ms {
                    if next.start_time >= data.start_time + constants.ln_end_threshold_ms {
                        if next.end_time > data.end_time + constants.ln_end_threshold_ms {
                            layer_type_update = LnLayerType::OutsideRelease;
                            multiplier_factor = constants.ln_release_after_multiplier;
                        } else if next.end_time > 0.0 {
                            layer_type_update = LnLayerType::InsideRelease;
                            multiplier_factor = constants.ln_release_before_multiplier;
                        } else {
                            layer_type_update = LnLayerType::InsideTap;
                            multiplier_factor = constants.ln_tap_multiplier;
                        }
                    }
                }
            }

            for (h_idx, _) in data.hit_objects.iter().enumerate() {
                let mut final_multiplier = base_multiplier;
                let mut final_type = LnLayerType::None;

                if layer_type_update != LnLayerType::None {
                    final_type = layer_type_update;
                    final_multiplier *= multiplier_factor;
                }

                updates.push((i, h_idx, final_multiplier, final_type));
            }
        }
    }

    // Apply updates
    for (d_idx, h_idx, mult, l_type) in updates {
        let obj = &mut strain_solver_data[d_idx].hit_objects[h_idx];
        obj.ln_strain_multiplier = mult;
        obj.ln_layer_type = l_type;
    }
}
