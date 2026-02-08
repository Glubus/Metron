use crate::quaver::quaver2025::difficulty::structs::{Hand, StrainSolverData};

/// Calculate overall difficulty of a map using the full algorithm with continuity adjustment
pub fn calculate_final_difficulty(
    strain_solver_data: &mut [StrainSolverData],
    use_fallback: bool,
) -> f64 {
    // When the map has only scratch key notes, StrainSolverData would be empty, so we return 0
    if strain_solver_data.is_empty() {
        return 0.0;
    }

    // Solve strain value of every data point
    for data in strain_solver_data.iter_mut() {
        data.calculate_strain_value();
    }

    let calculated_diff = strain_solver_data
        .iter()
        .filter(|s| matches!(s.hand, Hand::Left | Hand::Right))
        .map(|s| s.total_strain_value)
        .sum::<f64>()
        / strain_solver_data
            .iter()
            .filter(|s| matches!(s.hand, Hand::Left | Hand::Right))
            .count() as f64;

    // Determine map start and end
    let map_start = strain_solver_data
        .iter()
        .map(|s| s.start_time as i32)
        .min()
        .unwrap_or(0) as f64;

    let map_end = strain_solver_data
        .iter()
        .map(|s| s.end_time.max(s.start_time) as i32)
        .max()
        .unwrap_or(0) as f64;

    let bins = create_difficulty_bins(strain_solver_data, map_start, map_end, use_fallback);

    if !bins.iter().any(|&strain| strain > 0.0) {
        return 0.0;
    }

    let (continuity_adjustment, continuity) = calculate_continuity_adjustment(&bins);
    let short_map_adjustment = calculate_short_map_adjustment(&bins, continuity);

    calculated_diff * continuity_adjustment * short_map_adjustment
}

/// Create difficulty bins for analysis
fn create_difficulty_bins(
    strain_solver_data: &[StrainSolverData],
    map_start: f64,
    map_end: f64,
    use_fallback: bool,
) -> Vec<f64> {
    let mut bins = Vec::new();
    const BIN_SIZE: f64 = 1000.0;

    let mut left_index = 0;
    let mut right_index = 0;

    // Find starting index
    while left_index < strain_solver_data.len()
        && strain_solver_data[left_index].start_time < map_start
    {
        left_index += 1;
    }

    // We iterate with i32 steps to avoid float precision accumulation errors for the loop counter,
    // but we use floats for comparisons.
    let start_int = map_start as i32;
    let end_int = map_end as i32;
    let step_int = BIN_SIZE as i32;

    let mut current_time_int = start_int;
    while current_time_int < end_int {
        let bin_end = (current_time_int + step_int) as f64;

        let values_in_bin: Vec<&StrainSolverData> = if use_fallback {
            // Fallback for odd key counts: naive iteration
            let bin_start = current_time_int as f64;
            strain_solver_data
                .iter()
                .filter(|s| s.start_time >= bin_start && s.start_time < bin_end)
                .collect()
        } else {
            // Optimized binning for even key counts
            while right_index < strain_solver_data.len().saturating_sub(1)
                && strain_solver_data[right_index + 1].start_time < bin_end
            {
                right_index += 1;
            }

            if left_index >= strain_solver_data.len() {
                bins.push(0.0);
                current_time_int += step_int;
                continue;
            }

            // Slice from left to right (inclusive)
            if right_index >= left_index {
                strain_solver_data[left_index..=right_index]
                    .iter()
                    .collect()
            } else {
                Vec::new()
            }
        };

        let average_rating = if values_in_bin.is_empty() {
            0.0
        } else {
            values_in_bin
                .iter()
                .map(|s| s.total_strain_value)
                .sum::<f64>()
                / values_in_bin.len() as f64
        };

        bins.push(average_rating);

        if !use_fallback {
            left_index = right_index + 1;
        }
        current_time_int += step_int;
    }

    bins
}

/// Calculate continuity adjustment for difficulty
fn calculate_continuity_adjustment(bins: &[f64]) -> (f64, f64) {
    // Average of the hardest 40% of the map
    let cutoff_pos = (bins.len() as f64 * 0.4).floor() as usize;
    let mut sorted_bins = bins.to_vec();
    // Sort descending
    sorted_bins.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    let top_40 = &sorted_bins[..cutoff_pos];
    let easy_rating_cutoff = if top_40.is_empty() {
        0.0
    } else {
        top_40.iter().sum::<f64>() / top_40.len() as f64
    };

    // Calculate continuity - this should match the C# implementation exactly
    let continuity = if easy_rating_cutoff > 0.0 {
        let non_zero_bins: Vec<f64> = bins
            .iter()
            .filter(|&&strain| strain > 0.0)
            .map(|&strain| (strain / easy_rating_cutoff).sqrt())
            .collect();

        if !non_zero_bins.is_empty() {
            non_zero_bins.iter().sum::<f64>() / non_zero_bins.len() as f64
        } else {
            0.0
        }
    } else {
        0.0
    };

    // Apply continuity adjustment
    const MAX_CONTINUITY: f64 = 1.00;
    const AVG_CONTINUITY: f64 = 0.85;
    const MIN_CONTINUITY: f64 = 0.60;

    const MAX_ADJUSTMENT: f64 = 1.05;
    const AVG_ADJUSTMENT: f64 = 1.00;
    const MIN_ADJUSTMENT: f64 = 0.90;

    let continuity_adjustment = if continuity > AVG_CONTINUITY {
        let continuity_factor =
            1.0 - (continuity - AVG_CONTINUITY) / (MAX_CONTINUITY - AVG_CONTINUITY);
        (continuity_factor * (AVG_ADJUSTMENT - MIN_ADJUSTMENT) + MIN_ADJUSTMENT)
            .min(AVG_ADJUSTMENT)
            .max(MIN_ADJUSTMENT)
    } else {
        let continuity_factor =
            1.0 - (continuity - MIN_CONTINUITY) / (AVG_CONTINUITY - MIN_CONTINUITY);
        (continuity_factor * (MAX_ADJUSTMENT - AVG_ADJUSTMENT) + AVG_ADJUSTMENT)
            .min(MAX_ADJUSTMENT)
            .max(AVG_ADJUSTMENT)
    };

    (continuity_adjustment, continuity)
}

/// Calculate short map adjustment for difficulty
fn calculate_short_map_adjustment(bins: &[f64], continuity: f64) -> f64 {
    const MAX_SHORT_MAP_ADJUSTMENT: f64 = 0.75;
    const SHORT_MAP_THRESHOLD: f64 = 60.0 * 1000.0; // 60 seconds in milliseconds
    const BIN_SIZE: f64 = 1000.0;

    // Use the continuity value passed from the continuity adjustment calculation
    let true_drain_time = bins.len() as f64 * continuity * BIN_SIZE;
    let short_map_adjustment = (0.25 * (true_drain_time / SHORT_MAP_THRESHOLD).sqrt() + 0.75)
        .min(1.0)
        .max(MAX_SHORT_MAP_ADJUSTMENT);

    short_map_adjustment
}
