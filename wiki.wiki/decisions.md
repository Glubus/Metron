# Metron Decisions

## 2026-02-03: Core Calculator Trait

**Decision**: Implement `Calculator` trait and `Rating` output linkage as the core abstraction.
**Why**: To allow multiple VSRG rating algorithms to coexist and be hot-swappable (e.g., osu!, MinaCalc).
**Impact**: Defines the public API for all future rating implementations.
**Workflow**: `feature-add`

## 2026-02-03: Calculator Metadata

**Decision**: Add `name()`, `version()`, and `game()` methods to the `Calculator` trait.
**Why**: To allow self-describing calculators for better UI integration and multi-game support.
**Impact**: All future calculator implementations must provide this metadata.
**Workflow**: `feature-add`

## 2026-02-03: Single Assertion Principle

**Decision**: Enforce "One Assert Per Test" policy for unit tests.
**Why**: To ensure tests fail unambiguously and to improve diagnosis speed.
**Impact**: Tests must be granular. CI will eventually enforce this where possible (manual review for now).
**Workflow**: `rule-testing-pyramid`

## 2026-02-04: osu! 2015 Implementation

**Decision**: Implement the osu!mania 2015 algorithm as a reference calculator.
**Why**: To validate the `Calculator` trait with a concrete, real-world example.
**Impact**: Adds `src/osu/osu2015` module.
**Workflow**: `feature-add`

