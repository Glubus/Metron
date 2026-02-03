---
description: # Performance Investigation Workflow
---

## Purpose

Improve performance based on **evidence**, not intuition.

This workflow applies to:
- latency issues
- high CPU usage
- memory pressure
- scalability concerns

---

## Principles

- Measure before optimizing
- Change one variable at a time
- Always document results

---

## Steps

### 1. Establish a Baseline

Before changing anything:

- Measure current performance:
  - latency (p50 / p95 / p99)
  - CPU usage
  - memory usage
- Record the baseline values
 - Load Any Rules or Skills needed for resolving this matter

If possible:
- use production-like data
- run multiple samples

> No baseline = no optimization.

---

### 2. Formulate a Hypothesis

Explicitly state what you expect:

> “If we do **X**, then **Y** should improve because **Z**.”

Examples:
- Switching filter from Lanczos3 to Triangle should reduce CPU usage.
- Streaming instead of buffering should reduce peak memory.

Write the hypothesis down (comment or wiki).

---

### 3. Apply an Isolated Change

Rules:

- Change **only one thing**
- Avoid refactors or cleanups
- Keep the diff minimal

This ensures the result is attributable.

---

### 4. Measure Again

After the change:

- Re-run the same measurements
- Compare with the baseline
- Look for:
  - actual gains
  - regressions
  - trade-offs

---

### 5. Decide

Based on data:

- Keep the change if it improves the target metric
- Revert if gains are negligible or costs are too high

No emotional attachment to optimizations.

---

### 6. Document the Outcome

If the change is kept:

- Update `wiki.wiki`:
  - what was changed
  - measured impact
  - trade-offs

Example:
```markdown
### 2024-03-02: Image Resize Optimization
**Change**: Switched to streaming decode.
**Result**: -35% peak memory, +5% latency.
**Decision**: Accepted due to memory constraints.
````

---

## Exit Criteria

A performance investigation is complete only if:

* A baseline was measured
* A hypothesis was tested
* Results are documented
* The decision is justified by data