# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Refactor

- Refactored `quaver2025` difficulty calculation to use `f64` for higher precision.
- Split `quaver2025` difficulty logic into modular files (`clustering`, `fingering`, `manipulation`, `ln`, `difficulty`).
- Renamed `rate` to `clock_rate` in `QuaverDifficultyContext` and enforced usage of `ClockRate` type.

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
