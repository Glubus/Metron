use super::evaluators::Evaluator;
use rhythm_open_exchange::RoxChart;

pub struct Strain {
    pub(crate) individual: Vec<f64>,
    pub(crate) overall: f64,
    pub(crate) highest_individual: f64,

    pub(crate) peaks: Vec<f64>,
    pub(crate) current_section_peak: f64,
    pub(crate) current_section_end: f64,

    // Constants
    pub(crate) individual_decay_base: f64,
    pub(crate) overall_decay_base: f64,
    pub(crate) section_length: f64,
}

impl Strain {
    #[must_use]
    pub fn new(total_columns: usize) -> Self {
        Self {
            individual: vec![0.0; total_columns],
            overall: 1.0,
            highest_individual: 0.0,

            peaks: Vec::new(),
            current_section_peak: 0.0,
            current_section_end: 400.0,

            individual_decay_base: 0.125,
            overall_decay_base: 0.30,
            section_length: 400.0,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::too_many_arguments)]
    pub fn process(
        &mut self,
        chart: &RoxChart,
        current_idx: usize,
        current_time: f64,
        delta_time: f64,
        column_strain_time: f64,
        history: &[Option<usize>],
        clock_rate: f64,
    ) {
        // Section handling
        if current_time > self.current_section_end {
            self.save_current_peak();
            self.start_new_section_from(current_time);
        }

        // Object data
        let note = &chart.notes[current_idx];
        let duration = note.duration_us() as f64 / 1_000.0 / clock_rate;
        let end_time = current_time + duration;
        let col = note.column as usize;

        let safe_col = col.min(self.individual.len() - 1);

        // Strain logic

        // Individual Strain
        self.individual[safe_col] = self.apply_decay(
            self.individual[safe_col],
            column_strain_time,
            self.individual_decay_base,
        );
        self.individual[safe_col] += Evaluator::evaluate_individual(
            chart,
            current_idx,
            current_time,
            end_time,
            history,
            clock_rate,
        );

        // Highest individual logic
        if delta_time <= 1.0 {
            self.highest_individual = self.highest_individual.max(self.individual[safe_col]);
        } else {
            self.highest_individual = self.individual[safe_col];
        }

        // Overall Strain
        self.overall = self.apply_decay(self.overall, delta_time, self.overall_decay_base);
        self.overall += Evaluator::evaluate_overall(
            chart,
            current_idx,
            current_time,
            end_time,
            history,
            clock_rate,
        );

        let strain_value = self.highest_individual + self.overall;
        self.current_section_peak = self.current_section_peak.max(strain_value);
    }

    fn save_current_peak(&mut self) {
        self.peaks.push(self.current_section_peak);
    }

    fn start_new_section_from(&mut self, start_time: f64) {
        // Generic logic: advance section end until it covers current start
        while self.current_section_end < start_time {
            self.current_section_end += self.section_length;
        }

        self.current_section_peak = 0.0;
    }

    #[allow(clippy::unused_self)]
    fn apply_decay(&self, value: f64, delta_time: f64, decay_base: f64) -> f64 {
        value * decay_base.powf(delta_time / 1000.0)
    }

    pub fn difficulty_value(&mut self) -> f64 {
        self.save_current_peak(); // Save last section

        // Sort peaks descending
        self.peaks
            .sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        let mut difficulty = 0.0;
        let mut weight = 1.0;

        for strain in &self.peaks {
            difficulty += strain * weight;
            weight *= 0.9;
        }

        difficulty
    }
}
