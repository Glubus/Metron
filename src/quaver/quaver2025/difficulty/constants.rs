#[derive(Debug, Clone)]
pub struct StrainConstants {
    pub ln_end_threshold_ms: f64,
    pub chord_clump_tolerance_ms: f64,
    pub graph_interval_size_ms: i32,
    pub graph_interval_offset_ms: i32,

    // Simple Jacks
    pub s_jack_lower_boundary_ms: f64,
    pub s_jack_upper_boundary_ms: f64,
    pub s_jack_max_strain_value: f64,
    pub s_jack_curve_exponential: f64,

    // Tech Jacks
    pub t_jack_lower_boundary_ms: f64,
    pub t_jack_upper_boundary_ms: f64,
    pub t_jack_max_strain_value: f64,
    pub t_jack_curve_exponential: f64,

    // Rolls
    pub roll_lower_boundary_ms: f64,
    pub roll_upper_boundary_ms: f64,
    pub roll_max_strain_value: f64,
    pub roll_curve_exponential: f64,

    // Brackets
    pub bracket_lower_boundary_ms: f64,
    pub bracket_upper_boundary_ms: f64,
    pub bracket_max_strain_value: f64,
    pub bracket_curve_exponential: f64,

    // Roll Manipulation
    pub roll_ratio_tolerance_ms: f64,
    pub roll_ratio_multiplier: f64,
    pub roll_max_length: f64,
    pub roll_length_multiplier: f64,

    // Jack Manipulation (Vibro)
    pub vibro_action_duration_ms: f64,
    pub vibro_action_tolerance_ms: f64,
    pub vibro_multiplier: f64,
    pub vibro_length_multiplier: f64,
    pub vibro_max_length: f64,

    // LN Multipliers
    pub ln_base_multiplier: f64,
    pub ln_layer_tolerance_ms: f64,
    pub ln_layer_threshold_ms: f64,
    pub ln_release_after_multiplier: f64,
    pub ln_release_before_multiplier: f64,
    pub ln_tap_multiplier: f64,
}

impl Default for StrainConstants {
    fn default() -> Self {
        Self {
            ln_end_threshold_ms: 42.0,
            chord_clump_tolerance_ms: 8.0,
            graph_interval_size_ms: 500,
            graph_interval_offset_ms: 100,

            // Simple Jacks
            s_jack_lower_boundary_ms: 40.0,
            s_jack_upper_boundary_ms: 320.0,
            s_jack_max_strain_value: 68.0,
            s_jack_curve_exponential: 1.17,

            // Tech Jacks
            t_jack_lower_boundary_ms: 40.0,
            t_jack_upper_boundary_ms: 330.0,
            t_jack_max_strain_value: 70.0,
            t_jack_curve_exponential: 1.14,

            // Rolls
            roll_lower_boundary_ms: 30.0,
            roll_upper_boundary_ms: 230.0,
            roll_max_strain_value: 55.0,
            roll_curve_exponential: 1.13,

            // Brackets
            bracket_lower_boundary_ms: 30.0,
            bracket_upper_boundary_ms: 230.0,
            bracket_max_strain_value: 56.0,
            bracket_curve_exponential: 1.13,

            // Roll Manipulation
            roll_ratio_tolerance_ms: 2.0,
            roll_ratio_multiplier: 0.25,
            roll_length_multiplier: 0.6,
            roll_max_length: 14.0,

            // Jack Manipulation (Vibro)
            vibro_action_duration_ms: 88.2,
            vibro_action_tolerance_ms: 88.2,
            vibro_multiplier: 0.75,
            vibro_length_multiplier: 0.3,
            vibro_max_length: 6.0,

            // LN Multipliers
            ln_base_multiplier: 0.6,
            ln_layer_tolerance_ms: 60.0,
            ln_layer_threshold_ms: 93.7,
            ln_release_after_multiplier: 1.0,
            ln_release_before_multiplier: 1.3,
            ln_tap_multiplier: 1.05,
        }
    }
}
