# Metron Vision

## Core Purpose

**Metron** aims to be the "God Library" for VSRG (Vertical Scrolling Rhythm Game) difficulty and performance calculation.
It leverages the **ROX** format (`RoxChart`) as its universal input to calculate ratings for any chart type.

## Target Audience

- **Developers** building VSRG games, analysis tools, or competitive platforms.
- **Rhythm Game Enthusiasts** looking for transparent, high-performance rating algorithms.

## Key Goals

1. **Universal Input**: Strictly uses `RoxChart` as the input map format.
2. **Modular Architecture**:
    - `Rating` struct and **Traits** system.
    - Specific algorithms (e.g., osu!mania 2015, MinaCalc 5.15) enabled via **Cargo Features** to keep the core lightweight.
3. **Performance**: Blazing fast calculation suitable for real-time analysis or large-batch processing.

## Architecture Guidelines

- **Input**: `rox::RoxChart`
- **Output**: Flexible `RatingResult` types.
  - Must support diverse outputs (e.g., single `f32` for Star Rating, struct with 6+ values for MinaCalc MSD).
  - The `Rating` trait should define its specific output type.
- **Extensibility**: 3rd party developers should be able to implement the `Rating` trait for their own algorithms.

## Roadmap

1. **MVP**:
    - Define the `Rating` trait.
    - Implement one reference algorithm (e.g., simplistic density or a port of a known calc).
2. **Expansion**:
    - Port osu!mania 2015 rating.
    - Port MinaCalc 5.15.
3. **Optimization**:
    - Benchmarking and SIMD optimizations where applicable.
