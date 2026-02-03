# 2026-02-03: Core Calculator Trait
**Decision**: Implement `Calculator` trait and `Rating` output linkage as the core abstraction.
**Why**: To allow multiple VSRG rating algorithms to coexist and be hot-swappable (e.g., osu!, MinaCalc).
**Impact**: Defines the public API for all future rating implementations.
**Workflow**: `feature-add`
