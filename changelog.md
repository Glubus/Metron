# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0]

### Added

- **MinaCalc rows like Etterna** (`etterna::rows`, `Rows`, field `rows` of `MinaCalcDifficultyContext` and `MinaCalcPerformanceContext`, default `Rows::Quantized`): every chart reaches MinaCalc on Etterna's 1/48-beat rows, whatever its format. A chart whose notes already sit on rows (StepMania `.sm`/`.ssc` as ROX decodes them, stops and delays included) keeps its times; any other chart (osu! `.osu`, Quaver `.qua`, in raw milliseconds) is snapped onto rows the way Etterna's osu loader does it (rows counted from the first note, BPM breakpoints placed on rows, inherited points scaling the last BPM). The chart's format is not needed: a chart is on rows when at most 10 % of its note times are off the grid of their BPM segment (corpus: on-rows charts at most 2.2 %, osu charts at least 91.9 %). Notes that land on the same row merge into one MinaCalc row. `Rows::Exact` keeps the times as given (0.3.0 behaviour).
- **Rate mode** (`RateMode`, field `rate_mode` of `MinaCalcDifficultyContext`, default `RateMode::Exact`): `Exact` runs MinaCalc at the requested rate (the real value, and what Etterna does for a score's SSR); `Interpolated` reproduces Etterna's song-select value (`Steps::GetMSD`): the cached rates 0.7 to 2.0 as they are, the linear interpolation of the two neighbours between them, Etterna's extrapolation above 2.0 and below 0.7. It only runs MinaCalc at the one or two grid rates it needs.
- `MinaCalc527::rates_table`: the 14 cached rates (0.7x to 2.0x) of a chart in one `calc_all_rates` call, to store and interpolate later without the chart.
- Example `rows_compare`: prints, per chart file, whether `Rows::Quantized` moved any note and the MSD under both modes.

### Changed

- Default MSD of osu! and Quaver charts moves by the row snapping (reference osu! charts: 0.002 to 0.30 MSD, overall; StepMania charts unchanged). `minacalc-rs` 527.1, which no longer aborts on 7K rows with a middle-column note.

## [0.3.0]

### Changed

- **ROX 0.8**: Metron depends on `rhythm-open-exchange` 0.8, whose `RoxChart` is the one callers pass in (breaking: a chart from ROX 0.7 is another type). No calculator code or rating changes.

## [0.2.0]

### Changed

- **ROX from crates.io**: Metron depends on `rhythm-open-exchange` 0.7 (library `rox`, one crate with the `formats` feature) instead of the sibling `../Rhythm-Open-Exchange` path crates `rox` and `rox-formats`, so it builds alone and can be published. Imports are `rox::…` and `rox::formats::…` (breaking for code that named the chart types through Metron's old paths).
- **Etterna / MinaCalc 527**: `minacalc-rs` 527 (Etterna's current calculator). The module, the calculator and the example are renamed `minacalc527` / `MinaCalc527` / `minacalc527` (breaking); `VERSION` is `"527"`. 527 changes chordjack the most (a 4K chordjack reference chart: 40.56 in 515, 34.37 in 527).
- **MinaCalc input timed like Etterna**: `chart_to_notes` times every row from the chart's first row, as Etterna's `NoteData::SerializeNoteData2` does. MinaCalc cuts time into intervals counted from zero, so a chart's offset used to change its rating (up to 0.21 overall and 1.2 on a skillset on the reference charts). This also keeps every time non-negative.

### Fixed

- **MinaCalc safety**: a note column beyond the chart's key count (or 32) is refused instead of overflowing the row mask. `minacalc-rs` 527 also catches MinaCalc's C++ exceptions and refuses unsafe inputs.
- **Daniel**: key usage no longer indexes past its buffer for notes after the last calculation corner (it panicked on real charts).

### Added

- **Daniel Calculator**: Implemented a new custom osu!mania difficulty calculator (`src/custom/daniel`) based on the provided Daniel Python reference, while keeping `sunnyxxy` intact as a separate calculator.
  - Works on `RoxChart` input, not just `.osu` parsing.
  - Exposes star rating, smoothed difficulty graph, and the factor curves (`Pressing Intensity`, `Unevenness`, `Same-Column Pressure`, `Cross-Column Pressure`).
  - Supports `clock_rate` and `overall_difficulty` through `DanielDifficultyContext`.

### Performance

- **Daniel Optimization**: Reworked the Daniel calculator internals to reuse the allocation and cache-friendly patterns already proven in `sunnyxxy`, while keeping Daniel's own formulas and output intact.
  - Replaced several `Vec<Vec<_>>` intermediates with flat reusable buffers for key usage.
  - Switched active-column tracking to bitmasks.
  - Added `interp_values_into`, `step_interp_into`, and `smooth_on_corners_into` style reuse in the Daniel pipeline.
  - Refactored `jbar` and `xbar` to use scratch buffers instead of rebuilding multiple large temporary vectors on every run.

### Refactor

- **Daniel Structure**: Split Daniel into dedicated modules (`bars`, `calculations`, `interpolation`, `map_data`, `metrics`, `process`, `smoothing`) and reduced the responsibility of the main pipeline functions.

- **Etterna / MinaCalc 515**: Implemented the Etterna difficulty calculator (`src/etterna/minacalc515`) wrapping the `minacalc-rs` C++ bindings.
  - `calculate_difficulty` uses `CalcMode::Msd` (raw, uncapped — canonical rating as shown on etternaonline.com).
  - `calculate_performance` uses `CalcMode::Ssr` (score-relative, nerfed — capped at player accuracy).
  - Both modes are configurable via context (`MinaCalcDifficultyContext.mode`, `MinaCalcPerformanceContext.mode`).
  - `Calc` instance reused per thread via `thread_local!` to avoid re-allocation on every call.
  - Shared chart-to-notes conversion in `src/etterna/convert.rs` (reusable across future Etterna versions).

### Performance

- **Interlude 2025 Optimization**: Reduced allocation pressure and removed O(n²) patterns in difficulty calculation.
  - Replaced `BTreeMap<i64, Vec<...>>` grouping with a single `sort_unstable_by_key` pass.
  - Eliminated per-row `vec![0.0; key_count]` allocation inside the hot loop.
  - Replaced inverted column scan (`for k in 0..key_count { .any() }`) with direct note iteration.
  - Pre-allocated `strain_data_points` with `Vec::with_capacity`.
  - `weighted_overall_difficulty` now takes `Vec<f64>` by value to sort in-place (zero extra allocation).
  - Replaced stable `sort_by` with `sort_unstable_by`.
  - `0.5f64.ln()` replaced with `const LN_HALF = -LN_2`.
  - Eliminated duplicate `0.02 * delta_ms` computation in `ms_to_stream_bpm`.

### Refactor

- **Interlude 2025**: Extracted all loop bodies >3 lines into named functions for readability (`trill_contribution_for_hand`, `calculate_and_record_column_strain`, `sort_chart_notes`, `accumulate_weighted`).

### Refactor

- Refactored `quaver2025` difficulty calculation to use `f64` for higher precision.
- Split `quaver2025` difficulty logic into modular files (`clustering`, `fingering`, `manipulation`, `ln`, `difficulty`).
- Renamed `rate` to `clock_rate` in `QuaverDifficultyContext` and enforced usage of `ClockRate` type.

### Performance

- **Quaver 2025 Optimization**: Improved difficulty calculation speed by ~20x (0.5s for 10k charts vs ~11s).
  - Refactored clustering algorithm from O(N^2) to O(N) with correct interleaved note handling.
  - Eliminated intermediate vector allocations in binning logic.
  - Fused initialization and clustering passes to improve cache locality.

### Added

- **Quaver 2025 Calculator**: Implemented the Quaver 2025 difficulty calculator (`quaver2025`), porting the official C# logic to Rust with `f64` precision.
- **ClockRate Type**: Introduced dedicated `ClockRate` type in `src/clock_rate.rs` for type-safe clock rate handling across **all calculators**.
  - Encapsulates clock rate as percentage (100 = normal speed)
  - Implements `From<ClockRate> for f64` for seamless conversion to multiplier
  - Validates input range (1-1000%)
  - Provides `Default` trait for easy fallback to 100%
  - **Migration**: All calculator contexts (`Osu2018DifficultyContext`, `Interlude2025DifficultyContext`) now use `Option<ClockRate>` instead of raw `Option<u32>`
  - **Benefits**: Prevents accidental percentage/multiplier confusion, enables future precision improvements without API breakage
- **osu!mania 2016 Calculator**: Implemented the osu! 2016 performance points algorithm (`osu2016`), reusing 2018 difficulty calculation but with authentic 2016 scoring logic.
- **Score Input**: Added `score` field to `Osu2016PerformanceContext` to allow precise PP calculation based on score, which is the primary metric for 2016 pp.
- **Interlude 2025 Calculator**: Implemented the Interlude 2025 difficulty calculator port, including full strain and weighting logic.

- Core `Calculator` trait in `src/calculator.rs` defining the interface for rating algorithms.
- `Rating` trait for flexible output types.
- `RoxChart` input support via `rhythm-open-exchange`.
- Transformed project into a library (`lib.rs`).
- Added `name()`, `version()`, and `game()` metadata methods to `Calculator` trait.
- Added `CalculatorResult<T>` type alias for standard error handling.
- Hardened CI with `clippy::pedantic` lints in `justfile`.
- Enforced "Single Assertion Principle" in `wiki.wiki/decisions.md`.
- **Breaking**: Refactored `Calculator` trait to separate `Difficulty` and `Performance` calculation.
- Added `osu!mania 2018` calculator implementation (`src/osu/osu2018`).
- Fully ported `osu!mania 2018` difficulty (HitWindow + Strain) and performance logic.
- **Internal**: Refactored `osu!mania 2018` difficulty into a modular structure (`src/osu/osu2018/difficulty/`).
- **Internal**: Added unit tests for `evaluators.rs` to ensure strain calculation correctness.
