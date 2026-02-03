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
