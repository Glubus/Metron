use rhythm_open_exchange::RoxChart;
use super::evaluators::Evaluator;

pub struct Strain {
    pub(crate) individual_strains: Vec<f32>,
    pub(crate) overall_strain: f32,
    pub(crate) highest_individual_strain: f32,
    
    pub(crate) strain_peaks: Vec<f32>,
    pub(crate) current_section_peak: f32,
    pub(crate) current_section_end: f32,
    
    // Constants
    pub(crate) individual_decay_base: f32,
    pub(crate) overall_decay_base: f32,
    pub(crate) section_length: f32,
}

impl Strain {
    pub fn new(total_columns: usize) -> Self {
        Self {
            individual_strains: vec![0.0; total_columns],
            overall_strain: 1.0,
            highest_individual_strain: 0.0,
            
            strain_peaks: Vec::new(),
            current_section_peak: 0.0,
            current_section_end: 400.0,
            
            individual_decay_base: 0.125,
            overall_decay_base: 0.30,
            section_length: 400.0,
        }
    }

    pub fn process(
        &mut self, 
        chart: &RoxChart,
        current_idx: usize,
        current_time: f32,
        delta_time: f32,
        column_strain_time: f32,
        history: &[Option<usize>],
        clock_rate: f32
    ) {
        // Section handling
        if current_time > self.current_section_end {
            self.save_current_peak();
            self.start_new_section_from(current_time);
        }

        // Object data
        let note = &chart.notes[current_idx];
        let duration = note.duration_us() as f32 / 1_000.0 / clock_rate;
        let end_time = current_time + duration;
        let col = note.column as usize;
        
        let safe_col = col.min(self.individual_strains.len() - 1);

        // Strain logic
        
        // Individual Strain
        self.individual_strains[safe_col] = self.apply_decay(self.individual_strains[safe_col], column_strain_time, self.individual_decay_base);
        self.individual_strains[safe_col] += Evaluator::evaluate_individual(chart, current_idx, current_time, end_time, history, clock_rate);

        // Highest individual logic
        if delta_time <= 1.0 {
            self.highest_individual_strain = self.highest_individual_strain.max(self.individual_strains[safe_col]);
        } else {
            self.highest_individual_strain = self.individual_strains[safe_col];
        }

        // Overall Strain
        self.overall_strain = self.apply_decay(self.overall_strain, delta_time, self.overall_decay_base);
        self.overall_strain += Evaluator::evaluate_overall(chart, current_idx, current_time, end_time, history, clock_rate);

        let strain_value = self.highest_individual_strain + self.overall_strain;
        self.current_section_peak = self.current_section_peak.max(strain_value);
    }
    
    fn save_current_peak(&mut self) {
        self.strain_peaks.push(self.current_section_peak);
    }

    fn start_new_section_from(&mut self, start_time: f32) {
         // Generic logic: advance section end until it covers current start
        while self.current_section_end < start_time {
            self.current_section_end += self.section_length;
        }
        
        self.current_section_peak = 0.0;
    }

    fn apply_decay(&self, value: f32, delta_time: f32, decay_base: f32) -> f32 {
        value * decay_base.powf(delta_time / 1000.0)
    }
    
    pub fn difficulty_value(&mut self) -> f32 {
        self.save_current_peak(); // Save last section
        
        // Sort peaks descending
        self.strain_peaks.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        
        let mut difficulty = 0.0;
        let mut weight = 1.0;
        
        for strain in &self.strain_peaks {
            difficulty += strain * weight;
            weight *= 0.9;
        }
        
        difficulty
    }
}
