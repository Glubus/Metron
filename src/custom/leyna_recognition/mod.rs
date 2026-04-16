pub mod bpm;
pub mod grid;
pub mod merger;
pub mod timeline;
pub mod tree;
pub mod types;
pub mod window;

pub use bpm::TimingAnalyzer;
pub use grid::PatternGrid;
pub use timeline::{PatternTimeline, PatternTimelineEntry};
pub use tree::{QuadTreeBuilder, QuadTreeNode};
pub use types::{PatternCategory, PatternClassification, PatternType};
pub use window::CrossSegmentAnalyzer;

use rhythm_open_exchange::RoxChart;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, Deserialize)]
pub struct AnalysisResult {
    pub tree: Vec<QuadTreeNode>,
    pub timeline: PatternTimeline,
    pub key_count: u8,
}

impl Serialize for AnalysisResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AnalysisResult", 2)?;
        state.serialize_field("timeline", &self.timeline.entries)?;
        state.serialize_field("key_count", &self.key_count)?;
        state.end()
    }
}

pub fn analyze(chart: &RoxChart) -> AnalysisResult {
    let key_count = chart.key_count();
    let max_time_slices = 20;
    let ignore_holds = true;
    let window_size = 4;

    let (grids, timestamps) = PatternGrid::from_chart(chart, max_time_slices, ignore_holds);

    let mut trees = Vec::new();
    for grid in &grids {
        let builder = QuadTreeBuilder::new(grid);
        trees.push(builder.build());
    }

    let timing_analyzer = TimingAnalyzer::new(chart, ignore_holds);
    let cross_analyzer =
        CrossSegmentAnalyzer::new(&grids, &timestamps, &timing_analyzer, key_count as usize);
    let cross_results = cross_analyzer.analyze_cross_segment(window_size);

    let timeline = PatternTimeline::build_from_cross_analysis(
        &cross_results,
        &grids,
        &timestamps,
        key_count as usize,
    );

    AnalysisResult { tree: trees, timeline, key_count }
}
