use crate::quaver::quaver2025::difficulty::constants::StrainConstants;
use crate::quaver::quaver2025::difficulty::structs::StrainSolverData;

/// Consolidates individual notes into chords based on timing proximity.
///
/// This function iterates through the strain data and merges notes that occur
/// within the `chord_clump_tolerance_ms` window into a single `StrainSolverData` entry,
/// provided they are on the same hand.
pub fn process_clustering(
    strain_solver_data: &mut Vec<StrainSolverData>,
    constants: &StrainConstants,
) {
    let mut i = 0;
    while i < strain_solver_data.len().saturating_sub(1) {
        let mut j = i + 1;
        while j < strain_solver_data.len() {
            let ms_diff = strain_solver_data[j].start_time - strain_solver_data[i].start_time;

            // Check if next hit object is way past the tolerance
            if ms_diff > constants.chord_clump_tolerance_ms {
                break;
            }

            // Check if the next and current hit objects are chord-able
            if ms_diff.abs() <= constants.chord_clump_tolerance_ms {
                if strain_solver_data[i].hand == strain_solver_data[j].hand {
                    // Merge chord objects
                    let hit_objects_to_add = strain_solver_data[j].hit_objects.clone();

                    // Filter out duplicates based on finger state if necessary,
                    // though the original logic seemed to filter based on existing finger states.
                    // The original C# logic:
                    // foreach (var hitObject in StrainSolverData[j].HitObjects) ...

                    let mut objects_to_extend = Vec::new();
                    for hit_obj in hit_objects_to_add {
                        let same_state_found = strain_solver_data[i]
                            .hit_objects
                            .iter()
                            .any(|existing| existing.finger_state == hit_obj.finger_state);

                        if !same_state_found {
                            objects_to_extend.push(hit_obj);
                        }
                    }

                    strain_solver_data[i].hit_objects.extend(objects_to_extend);
                    strain_solver_data.remove(j);
                    continue;
                }
            }
            j += 1;
        }
        i += 1;
    }

    // Solve finger state of every object once chords have been found and applied
    for data in strain_solver_data.iter_mut() {
        data.solve_finger_state();
    }
}
