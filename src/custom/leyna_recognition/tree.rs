use super::grid::PatternGrid;
use super::types::{PatternCategory, PatternClassification, PatternType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadTreeNode {
    pub time_start: usize,
    pub time_end: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub classification: PatternClassification,
    pub merged_classification: Option<PatternType>,
    pub children: Vec<QuadTreeNode>,
}

pub struct QuadTreeBuilder<'a> {
    grid: &'a PatternGrid,
}

impl<'a> QuadTreeBuilder<'a> {
    pub fn new(grid: &'a PatternGrid) -> Self {
        Self { grid }
    }

    pub fn build(&self) -> QuadTreeNode {
        self.build_node(0, self.grid.time_slices, 0, self.grid.columns)
    }

    fn build_node(&self, time_start: usize, time_end: usize, col_start: usize, col_end: usize) -> QuadTreeNode {
        let width = col_end - col_start;
        let height = time_end - time_start;

        if width <= 2 && height <= 2 {
            return self.create_leaf_node(time_start, time_end, col_start, col_end);
        }

        let mut time_mid = time_start + height / 2;
        let mut col_mid = col_start + width / 2;
        if time_mid == time_start { time_mid = time_start + 1; }
        if col_mid == col_start { col_mid = col_start + 1; }

        let mut children = Vec::new();
        if time_mid > time_start && col_mid > col_start {
            children.push(self.build_node(time_start, time_mid, col_start, col_mid));
        }
        if time_mid > time_start && col_end > col_mid {
            children.push(self.build_node(time_start, time_mid, col_mid, col_end));
        }
        if time_end > time_mid && col_mid > col_start {
            children.push(self.build_node(time_mid, time_end, col_start, col_mid));
        }
        if time_end > time_mid && col_end > col_mid {
            children.push(self.build_node(time_mid, time_end, col_mid, col_end));
        }

        if children.len() <= 1 {
            return self.create_leaf_node(time_start, time_end, col_start, col_end);
        }

        let merged = self.merge_classifications(&children);
        let classification = self.compute_aggregate_classification(&children);

        QuadTreeNode { time_start, time_end, col_start, col_end, classification, merged_classification: merged, children }
    }

    fn create_leaf_node(&self, time_start: usize, time_end: usize, col_start: usize, col_end: usize) -> QuadTreeNode {
        let tl = self.grid.get_note(time_start, col_start);
        let tr = if col_start + 1 < self.grid.columns { self.grid.get_note(time_start, col_start + 1) } else { false };
        let bl = if time_start + 1 < self.grid.time_slices { self.grid.get_note(time_start + 1, col_start) } else { false };
        let br = if time_start + 1 < self.grid.time_slices && col_start + 1 < self.grid.columns {
            self.grid.get_note(time_start + 1, col_start + 1)
        } else { false };
        let classification = PatternClassification::from_grid(tl, tr, bl, br);
        QuadTreeNode { time_start, time_end, col_start, col_end, classification, merged_classification: None, children: Vec::new() }
    }

    fn merge_classifications(&self, children: &[QuadTreeNode]) -> Option<PatternType> {
        if children.is_empty() { return None; }
        let classifications: Vec<_> = children.iter().map(|c| c.classification).collect();
        let categories: Vec<_> = classifications.iter().map(|c| c.get_category()).collect();

        if children.len() == 4 {
            let key = (classifications[0], classifications[1], classifications[2], classifications[3]);
            match key {
                (PatternClassification::Empty, PatternClassification::Empty,
                 PatternClassification::Empty, PatternClassification::Empty) => return Some(PatternType::EmptyRegion),
                (PatternClassification::TrillDown, PatternClassification::TrillDown,
                 PatternClassification::TrillDown, PatternClassification::TrillDown) => return Some(PatternType::Stream),
                (PatternClassification::TrillUp, PatternClassification::TrillUp,
                 PatternClassification::TrillUp, PatternClassification::TrillUp) => return Some(PatternType::ReverseStream),
                (PatternClassification::JackLeft, PatternClassification::JackLeft,
                 PatternClassification::JackLeft, PatternClassification::JackLeft) => return Some(PatternType::ExtendedJackLeft),
                (PatternClassification::JackRight, PatternClassification::JackRight,
                 PatternClassification::JackRight, PatternClassification::JackRight) => return Some(PatternType::ExtendedJackRight),
                (PatternClassification::JackLeft, PatternClassification::JackRight,
                 PatternClassification::JackLeft, PatternClassification::JackRight)
                | (PatternClassification::JackRight, PatternClassification::JackLeft,
                   PatternClassification::JackRight, PatternClassification::JackLeft) => return Some(PatternType::SplitJack),
                (PatternClassification::JumpTop, PatternClassification::JumpTop,
                 PatternClassification::JumpTop, PatternClassification::JumpTop)
                | (PatternClassification::JumpBottom, PatternClassification::JumpBottom,
                   PatternClassification::JumpBottom, PatternClassification::JumpBottom) => return Some(PatternType::JumpSection),
                (PatternClassification::JumpTop, PatternClassification::JumpBottom,
                 PatternClassification::JumpTop, PatternClassification::JumpBottom)
                | (PatternClassification::JumpBottom, PatternClassification::JumpTop,
                   PatternClassification::JumpBottom, PatternClassification::JumpTop) => return Some(PatternType::AlternatingJumps),
                (PatternClassification::Chord, PatternClassification::Chord,
                 PatternClassification::Chord, PatternClassification::Chord) => return Some(PatternType::ChordSection),
                (PatternClassification::SingleTL, PatternClassification::SingleTR,
                 PatternClassification::SingleBL, PatternClassification::SingleBR)
                | (PatternClassification::SingleTL, PatternClassification::SingleBR,
                   PatternClassification::SingleTR, PatternClassification::SingleBL) => return Some(PatternType::Scattered),
                _ => {}
            }
        }

        let unique_categories: HashSet<_> = categories.iter().cloned().collect();
        let pattern_type = if unique_categories.len() == 1 {
            match unique_categories.iter().next().unwrap() {
                PatternCategory::Empty => PatternType::EmptyRegion,
                PatternCategory::Single => PatternType::SingleNotes,
                PatternCategory::Jump => PatternType::JumpSection,
                PatternCategory::Jack => PatternType::JackSection,
                PatternCategory::Trill => PatternType::StreamSection,
                PatternCategory::Chord => PatternType::ChordSection,
                PatternCategory::Triple => PatternType::TripleSection,
            }
        } else if unique_categories.len() == 2 && unique_categories.contains(&PatternCategory::Empty) {
            if unique_categories.contains(&PatternCategory::Single) { PatternType::SparseSingles }
            else if unique_categories.contains(&PatternCategory::Jump) { PatternType::SparseJumps }
            else if unique_categories.contains(&PatternCategory::Jack) { PatternType::SparseJacks }
            else if unique_categories.contains(&PatternCategory::Trill) { PatternType::SparseStream }
            else if unique_categories.contains(&PatternCategory::Chord) { PatternType::SparseChords }
            else { PatternType::Mixed }
        } else if unique_categories.len() == 2 && unique_categories.contains(&PatternCategory::Single) {
            if unique_categories.contains(&PatternCategory::Jump) { PatternType::JumpWithSingles }
            else if unique_categories.contains(&PatternCategory::Jack) { PatternType::JackWithSingles }
            else if unique_categories.contains(&PatternCategory::Trill) { PatternType::StreamWithSingles }
            else if unique_categories.contains(&PatternCategory::Chord) { PatternType::ChordWithSingles }
            else if unique_categories.contains(&PatternCategory::Triple) { PatternType::TripleWithSingles }
            else { PatternType::Mixed }
        } else if unique_categories.contains(&PatternCategory::Trill) && unique_categories.contains(&PatternCategory::Jump) {
            PatternType::Jumpstream
        } else if unique_categories.contains(&PatternCategory::Chord)
            && (unique_categories.contains(&PatternCategory::Jump) || unique_categories.contains(&PatternCategory::Jack))
        {
            PatternType::Chordjack
        } else {
            self.dynamic_classification(children, &classifications, &categories)
        };

        Some(self.qualify_merge_name(pattern_type, children, &categories))
    }

    fn qualify_merge_name(&self, base_type: PatternType, children: &[QuadTreeNode], categories: &[PatternCategory]) -> PatternType {
        let n = children.len();
        let note_count: u32 = children.iter().map(|c| c.classification.note_count()).sum();
        let empty_count = categories.iter().filter(|&&c| c == PatternCategory::Empty).count();
        let single_count = categories.iter().filter(|&&c| c == PatternCategory::Single).count();
        let density = note_count as f64 / (n as f64 * 4.0);

        if density >= 0.75 {
            match base_type {
                PatternType::Stream | PatternType::StreamSection => PatternType::StreamDense,
                PatternType::Jumpstream => PatternType::JumpstreamDense,
                PatternType::Handstream => PatternType::HandstreamDense,
                PatternType::Chordjack => PatternType::ChordjackDense,
                PatternType::JumpSection => PatternType::DenseJumps,
                PatternType::ChordSection => PatternType::DenseChord,
                _ => base_type,
            }
        } else if density <= 0.25 && empty_count > 0 {
            match base_type {
                PatternType::Stream | PatternType::StreamSection => PatternType::SparseStream,
                PatternType::JumpSection => PatternType::SparseJumps,
                PatternType::JackSection => PatternType::SparseJacks,
                PatternType::ChordSection => PatternType::SparseChords,
                _ => base_type,
            }
        } else if single_count > n / 2 {
            match base_type {
                PatternType::Stream | PatternType::StreamSection => PatternType::StreamWithSingles,
                PatternType::Jumpstream => PatternType::JumpstreamWithSingles,
                PatternType::JumpSection => PatternType::JumpWithSingles,
                PatternType::JackSection => PatternType::JackWithSingles,
                PatternType::ChordSection => PatternType::ChordWithSingles,
                PatternType::TripleSection => PatternType::TripleWithSingles,
                _ => base_type,
            }
        } else {
            base_type
        }
    }

    fn dynamic_classification(&self, children: &[QuadTreeNode], classifications: &[PatternClassification], categories: &[PatternCategory]) -> PatternType {
        let n = children.len();
        let mut cat_counts = HashMap::new();
        for &cat in categories { *cat_counts.entry(cat).or_insert(0) += 1; }

        let empty_count = *cat_counts.get(&PatternCategory::Empty).unwrap_or(&0);
        let note_count: u32 = classifications.iter().map(|c| c.note_count()).sum();

        if empty_count == n { return PatternType::EmptyRegion; }
        if empty_count >= (n as f64 * 0.75) as usize { return PatternType::VerySparse; }

        let non_empty_counts: HashMap<_, _> = cat_counts.iter()
            .filter(|&(&cat, _)| cat != PatternCategory::Empty)
            .collect();

        if !non_empty_counts.is_empty() {
            let (&dominant_cat, dominant_count) = non_empty_counts.iter()
                .max_by_key(|&(_, &count)| count).unwrap();
            let dominance_ratio = **dominant_count as f64 / n as f64;

            if dominance_ratio >= 0.5 {
                let single_count = *cat_counts.get(&PatternCategory::Single).unwrap_or(&0);
                if single_count > 0 && single_count < n / 2 {
                    return match dominant_cat {
                        PatternCategory::Trill => PatternType::StreamWithSingles,
                        PatternCategory::Jump => PatternType::JumpWithSingles,
                        PatternCategory::Jack => PatternType::JackWithSingles,
                        PatternCategory::Chord => PatternType::ChordWithSingles,
                        _ => PatternType::Mixed,
                    };
                }
                if empty_count > 0 {
                    return match dominant_cat {
                        PatternCategory::Trill => PatternType::SparseStream,
                        PatternCategory::Jump => PatternType::SparseJumps,
                        PatternCategory::Jack => PatternType::SparseJacks,
                        PatternCategory::Chord => PatternType::SparseChords,
                        _ => PatternType::Mixed,
                    };
                }
                return match dominant_cat {
                    PatternCategory::Trill => PatternType::Stream,
                    PatternCategory::Jump => PatternType::JumpSection,
                    PatternCategory::Jack => PatternType::JackSection,
                    PatternCategory::Chord => PatternType::ChordSection,
                    PatternCategory::Single => PatternType::SingleNotes,
                    PatternCategory::Triple => PatternType::TripleSection,
                    _ => PatternType::Mixed,
                };
            }
        }

        let unique_count = non_empty_counts.len();
        if unique_count >= 3 {
            if note_count >= (n * 3) as u32 { PatternType::ComplexDense } else { PatternType::ComplexMixed }
        } else if unique_count == 2 {
            PatternType::Mixed
        } else {
            let density = note_count as f64 / (n as f64 * 4.0);
            if density >= 0.6 { PatternType::Dense }
            else if density >= 0.3 { PatternType::Moderate }
            else if density > 0.0 { PatternType::Light }
            else { PatternType::Mixed }
        }
    }

    fn compute_aggregate_classification(&self, children: &[QuadTreeNode]) -> PatternClassification {
        let classifications: Vec<_> = children.iter().map(|c| c.classification).collect();
        if classifications.iter().all(|&c| c == classifications[0]) { return classifications[0]; }
        let non_empty: Vec<_> = classifications.iter().filter(|&&c| c != PatternClassification::Empty).collect();
        if !non_empty.is_empty() { return **non_empty.iter().max_by_key(|c| c.note_count()).unwrap(); }
        *classifications.iter().max_by_key(|c| c.note_count()).unwrap()
    }
}
