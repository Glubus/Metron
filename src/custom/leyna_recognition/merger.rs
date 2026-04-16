use super::types::PatternType;

pub struct PatternMerger;

impl PatternMerger {
    pub fn are_compatible(p1: PatternType, p2: PatternType) -> bool {
        if p1 as u8 == p2 as u8 { return true; }

        let stream_group = [
            PatternType::Stream, PatternType::Light, PatternType::Mixed,
            PatternType::SparseStream, PatternType::StreamSection,
            PatternType::Jumpstream, PatternType::JumpSection,
            PatternType::Handstream, PatternType::TechnicalHybrid,
        ];
        let chordjack_group = [
            PatternType::Chordjack, PatternType::JackSection, PatternType::DenseChord,
        ];

        let in_stream_1 = stream_group.contains(&p1);
        let in_stream_2 = stream_group.contains(&p2);
        if in_stream_1 && in_stream_2 { return true; }

        let in_cj_1 = chordjack_group.contains(&p1);
        let in_cj_2 = chordjack_group.contains(&p2);
        if in_cj_1 && in_cj_2 { return true; }

        false
    }

    pub fn get_dominant_pattern(p1: PatternType, p2: PatternType) -> PatternType {
        let p1_priority = Self::get_priority(p1);
        let p2_priority = Self::get_priority(p2);
        if p1_priority >= p2_priority { p1 } else { p2 }
    }

    fn get_priority(p: PatternType) -> i32 {
        match p {
            PatternType::DenseChord => 100,
            PatternType::Chordjack => 90,
            PatternType::Handstream => 80,
            PatternType::Jumpstream => 70,
            PatternType::Stream => 60,
            PatternType::JackSection => 50,
            PatternType::JumpSection => 40,
            PatternType::TechnicalHybrid => 35,
            PatternType::Mixed => 20,
            PatternType::Light => 10,
            PatternType::EmptyRegion => 0,
            _ => 15,
        }
    }
}
