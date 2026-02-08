use crate::calculator::Rating;

#[derive(Debug, Clone)]
pub struct QuaverDifficulty {
    pub stars: f64,
}

impl Rating for QuaverDifficulty {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Left,
    Right,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FingerAction {
    Roll,
    SimpleJack,
    TechnicalJack,
    Bracket,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LnLayerType {
    None,
    InsideTap,
    InsideRelease,
    OutsideRelease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FingerState(pub u32);

impl FingerState {
    pub const NONE: Self = Self(0);
    pub const THUMB: Self = Self(1 << 31); // Arbitrary high bit for thumb if needed

    pub fn from_bits(bits: u32) -> Option<Self> {
        Some(Self(bits))
    }

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitAnd for FingerState {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

#[derive(Debug, Clone)]
pub struct StrainSolverHitObject {
    pub start_time: f64, // ms
    pub end_time: f64,   // ms
    pub lane: i32,
    pub finger_state: FingerState,
    pub ln_strain_multiplier: f64,
    pub ln_layer_type: LnLayerType,
}

impl StrainSolverHitObject {
    pub fn new(start_time: f64, lane: i32) -> Self {
        Self {
            start_time,
            end_time: 0.0,
            lane,
            finger_state: FingerState::NONE,
            ln_strain_multiplier: 1.0,
            ln_layer_type: LnLayerType::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StrainSolverData {
    pub hit_objects: Vec<StrainSolverHitObject>,
    pub start_time: f64,
    pub end_time: f64,
    pub hand: Hand,
    pub finger_state: FingerState,
    pub finger_action: FingerAction,
    pub finger_action_duration_ms: f64,
    pub action_strain_coefficient: f64,
    pub roll_manipulation_strain_multiplier: f64,
    pub total_strain_value: f64,

    // We use index to model the recursive "Next" pointer to avoid stale clones
    pub next_strain_solver_index_on_current_hand: Option<usize>,
}

impl StrainSolverData {
    pub fn new(hit_object: StrainSolverHitObject) -> Self {
        Self {
            start_time: hit_object.start_time,
            end_time: hit_object.end_time,
            hit_objects: vec![hit_object],
            hand: Hand::Ambiguous,
            finger_state: FingerState::NONE,
            finger_action: FingerAction::None,
            finger_action_duration_ms: 0.0,
            action_strain_coefficient: 1.0,
            roll_manipulation_strain_multiplier: 1.0,
            total_strain_value: 0.0,
            next_strain_solver_index_on_current_hand: None,
        }
    }

    pub fn solve_finger_state(&mut self) {
        let mut state = 0;
        for obj in &self.hit_objects {
            state |= obj.finger_state.0;
        }
        self.finger_state = FingerState(state);
    }

    pub fn hand_chord(&self) -> bool {
        self.hit_objects.len() > 1
    }

    pub fn calculate_strain_value(&mut self) {
        self.total_strain_value =
            self.action_strain_coefficient * self.roll_manipulation_strain_multiplier;

        let mut ln_multiplier = 0.0;
        for obj in &self.hit_objects {
            ln_multiplier += obj.ln_strain_multiplier;
        }

        if !self.hit_objects.is_empty() {
            self.total_strain_value *= ln_multiplier / self.hit_objects.len() as f64;
        }
    }
}
