use serde::{Deserialize, Serialize};

/// Named classifications for 2x2 cell patterns.
///
/// Grid layout: [TL][TR] (top = earlier time)
///              [BL][BR] (bottom = later time)
/// Binary encoding: TL TR BL BR (4 bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PatternClassification {
    Empty = 0b0000,
    SingleTL = 0b1000,
    SingleTR = 0b0100,
    SingleBL = 0b0010,
    SingleBR = 0b0001,
    JumpTop = 0b1100,
    JumpBottom = 0b0011,
    JackLeft = 0b1010,
    JackRight = 0b0101,
    TrillDown = 0b1001,
    TrillUp = 0b0110,
    TripleNoTL = 0b0111,
    TripleNoTR = 0b1011,
    TripleNoBL = 0b1101,
    TripleNoBR = 0b1110,
    Chord = 0b1111,
}

impl PatternClassification {
    pub fn from_grid(tl: bool, tr: bool, bl: bool, br: bool) -> Self {
        let binary = (if tl { 8 } else { 0 })
            | (if tr { 4 } else { 0 })
            | (if bl { 2 } else { 0 })
            | (if br { 1 } else { 0 });
        match binary {
            0b0000 => Self::Empty,
            0b1000 => Self::SingleTL,
            0b0100 => Self::SingleTR,
            0b0010 => Self::SingleBL,
            0b0001 => Self::SingleBR,
            0b1100 => Self::JumpTop,
            0b0011 => Self::JumpBottom,
            0b1010 => Self::JackLeft,
            0b0101 => Self::JackRight,
            0b1001 => Self::TrillDown,
            0b0110 => Self::TrillUp,
            0b0111 => Self::TripleNoTL,
            0b1011 => Self::TripleNoTR,
            0b1101 => Self::TripleNoBL,
            0b1110 => Self::TripleNoBR,
            0b1111 => Self::Chord,
            _ => unreachable!(),
        }
    }

    pub fn note_count(&self) -> u32 {
        (*self as u8).count_ones()
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::Empty
    }

    pub fn has_jump(&self) -> bool {
        matches!(
            self,
            Self::JumpTop | Self::JumpBottom | Self::TripleNoTL | Self::TripleNoTR
                | Self::TripleNoBL | Self::TripleNoBR | Self::Chord
        )
    }

    pub fn has_jack(&self) -> bool {
        matches!(
            self,
            Self::JackLeft | Self::JackRight | Self::TripleNoTL | Self::TripleNoTR
                | Self::TripleNoBL | Self::TripleNoBR | Self::Chord
        )
    }

    pub fn get_category(&self) -> PatternCategory {
        match self {
            Self::Empty => PatternCategory::Empty,
            Self::SingleTL | Self::SingleTR | Self::SingleBL | Self::SingleBR => PatternCategory::Single,
            Self::JumpTop | Self::JumpBottom => PatternCategory::Jump,
            Self::JackLeft | Self::JackRight => PatternCategory::Jack,
            Self::TrillDown | Self::TrillUp => PatternCategory::Trill,
            Self::TripleNoTL | Self::TripleNoTR | Self::TripleNoBL | Self::TripleNoBR => PatternCategory::Triple,
            Self::Chord => PatternCategory::Chord,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PatternCategory {
    Empty, Single, Jump, Jack, Trill, Triple, Chord,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatternType {
    EmptyRegion, VerySparse,
    SingleNotes, Scattered, SparseSingles,
    Stream, ReverseStream, StreamSection, SparseStream, StreamWithSingles, StreamDense,
    JumpSection, SparseJumps, JumpWithSingles, LightJumps, DenseJumps, AlternatingJumps,
    JackSection, ExtendedJackLeft, ExtendedJackRight, SplitJack, SparseJacks, JackWithSingles, LightJacks,
    ChordSection, SparseChords, ChordWithSingles, LightChords, DenseChord, TripleSection, TripleWithSingles,
    TechnicalHybrid, TechnicalWithSingles, SparseTechnical,
    Jumpstream, JumpstreamDense, JumpstreamWithSingles,
    Handstream, HandstreamDense,
    Chordjack, ChordjackDense,
    Mixed, ComplexMixed, ComplexDense, Dense, Moderate, Light,
}

impl PatternType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EmptyRegion => "EmptyRegion",
            Self::VerySparse => "VerySparse",
            Self::SingleNotes => "SingleNotes",
            Self::Scattered => "Scattered",
            Self::SparseSingles => "SparseSingles",
            Self::Stream => "Stream",
            Self::ReverseStream => "ReverseStream",
            Self::StreamSection => "StreamSection",
            Self::SparseStream => "SparseStream",
            Self::StreamWithSingles => "StreamWithSingles",
            Self::StreamDense => "StreamDense",
            Self::JumpSection => "JumpSection",
            Self::SparseJumps => "SparseJumps",
            Self::JumpWithSingles => "JumpWithSingles",
            Self::LightJumps => "LightJumps",
            Self::DenseJumps => "DenseJumps",
            Self::AlternatingJumps => "AlternatingJumps",
            Self::JackSection => "JackSection",
            Self::ExtendedJackLeft => "ExtendedJackLeft",
            Self::ExtendedJackRight => "ExtendedJackRight",
            Self::SplitJack => "SplitJack",
            Self::SparseJacks => "SparseJacks",
            Self::JackWithSingles => "JackWithSingles",
            Self::LightJacks => "LightJacks",
            Self::ChordSection => "ChordSection",
            Self::SparseChords => "SparseChords",
            Self::ChordWithSingles => "ChordWithSingles",
            Self::LightChords => "LightChords",
            Self::DenseChord => "DenseChord",
            Self::TripleSection => "TripleSection",
            Self::TripleWithSingles => "TripleWithSingles",
            Self::TechnicalHybrid => "TechnicalHybrid",
            Self::TechnicalWithSingles => "TechnicalWithSingles",
            Self::SparseTechnical => "SparseTechnical",
            Self::Jumpstream => "Jumpstream",
            Self::JumpstreamDense => "JumpstreamDense",
            Self::JumpstreamWithSingles => "JumpstreamWithSingles",
            Self::Handstream => "Handstream",
            Self::HandstreamDense => "HandstreamDense",
            Self::Chordjack => "Chordjack",
            Self::ChordjackDense => "ChordjackDense",
            Self::Mixed => "Mixed",
            Self::ComplexMixed => "ComplexMixed",
            Self::ComplexDense => "ComplexDense",
            Self::Dense => "Dense",
            Self::Moderate => "Moderate",
            Self::Light => "Light",
        }
    }
}
