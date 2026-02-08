# Metron Decisions

## 2026-02-03: Core Calculator Trait

**Decision**: Implement `Calculator` trait and `Rating` output linkage as the core abstraction.
**Why**: To allow multiple VSRG rating algorithms to coexist and be hot-swappable (e.g., osu!, MinaCalc).
**Impact**: Defines the public API for all future rating implementations.

## 2026-02-03: Calculator Metadata

**Decision**: Add `name()`, `version()`, and `game()` methods to the `Calculator` trait.
**Why**: To allow self-describing calculators for better UI integration and multi-game support.
**Impact**: All future calculator implementations must provide this metadata.

## 2026-02-03: Single Assertion Principle

**Decision**: Enforce "One Assert Per Test" policy for unit tests.
**Why**: To ensure tests fail unambiguously and to improve diagnosis speed.
**Impact**: Tests must be granular. CI will eventually enforce this where possible (manual review for now).

## 2026-02-04: osu! 2015 Implementation

**Decision**: Implement the osu!mania 2015 algorithm as a reference calculator.
**Why**: To validate the `Calculator` trait with a concrete, real-world example.
**Impact**: Adds `src/osu/osu2015` module.

## 2026-02-04: osu! 2018 Difficulty Refactor

**Decision**: Refactor `osu2018/difficulty.rs` into a modular structure (`difficulty/` folder).
**Why**: To improve maintainability, separation of concerns (Strain vs Evaluators), and testability. The current file is becoming too large.
**Impact**: Internal organizational change for `osu2018` module.

## 2026-02-08: ClockRate Type - Global Adoption

**Decision**: Create dedicated `ClockRate` type in `src/clock_rate.rs` and migrate **all calculators** to use it instead of raw `u32` or `f32` for clock rate values.

**Why**:

- **Type Safety Across Codebase**: Using raw integers led to confusion between percentages (100 = 1.0x) and direct multipliers. A calculator using 150 could mistakenly be interpreted as 150x instead of 1.5x.
- **Encapsulation**: Internal representation (percentage) is hidden, allowing future precision changes without API breakage across all calculators
- **Validation at Boundaries**: Enforces valid range (1-1000%) at construction time, preventing invalid clock rates from propagating
- **Consistent API**: All calculators (`osu2018`, `osu2016`, `interlude2025`) now have identical clock rate handling
- **Evolution Path**: `From<ClockRate> for f64` trait enables seamless integration while preserving ability to change precision later (e.g., switching to sub-percentage precision for frame-perfect timing)

**Impact**:

- **Breaking Change**: All calculator contexts migrated from `Option<u32>` to `Option<ClockRate>`
  - `Osu2018DifficultyContext`
  - `Interlude2025DifficultyContext`
  - All future calculator contexts
- **Test Updates**: All tests updated to use `ClockRate::from_percentage()`
- **Conversion Pattern**: Established standard pattern `f64::from(clock_rate.unwrap_or_default())` for difficulty calculations
- **Future-Proof**: Internal precision improvements (e.g., switching to f64 internally, supporting sub-percentage values) won't affect consumer code

**Why Global Adoption Matters**:

1. **Consistency**: A developer using multiple calculators expects the same API
2. **Maintainability**: No mixing of raw integers and typed values across the codebase
3. **Safety**: Type system prevents `let rate: u32 = 150` from being confused with `let multiplier: f64 = 150.0`
4. **Documentation**: The type itself documents what the value represents
