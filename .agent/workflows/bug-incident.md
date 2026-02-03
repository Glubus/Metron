---
description: Bug & Incident Resolution Workflow
---

## Purpose

Fix bugs and production incidents:
- without regressions
- with a clear understanding of the root cause
- while preserving long-term knowledge

This workflow applies to:
- production incidents
- critical bugs
- hard-to-reproduce issues

---

## Steps

### 1. Reproduce the Issue

Before attempting any fix:

- Reproduce the bug locally or in a controlled environment
- Create a **minimal failing test** if possible
- Prefer:
  - unit tests for logic bugs
  - property-based tests for edge cases

> If the bug cannot be reproduced, do not fix blindly.

---

### 2. Identify and Record the Root Cause

Once reproduced:

- Identify **why** the bug occurred (not just where)
- Record the root cause in `wiki.wiki`:
  - faulty assumption
  - missing validation
  - race condition
  - external dependency behavior

Example:
```markdown
### 2024-02-12: Corrupted WebP Crash
**Root Cause**: Missing validation on decoded image dimensions.
**Impact**: Panic during resize under malformed input.
````

---

### 3. Implement the Fix

Rules:

* Fix the **cause**, not the symptom
* Keep the change minimal
* Avoid unrelated refactors

If refactoring is required:

* justify it explicitly in the commit message or wiki

---

### 4. Add Regression Protection

After the fix:

* Add or update tests to prevent regression
* Ensure the test fails without the fix
* Ensure the test passes with the fix

> A bug fix without a regression test is incomplete.

---

### 5. Commit the Fix

Use Conventional Commits:

```bash
git commit -m "fix: US221-handle-corrupted-webp-input"
```

Rules:

* One bug per commit
* No opportunistic changes

---

### 6. Post-Fix Verification

Before closing the incident:

* Run the full test suite
* Verify related edge cases
* Confirm no performance regression

---

## Exit Criteria

A bug or incident is resolved only if:

* The issue is reproducible and fixed
* A regression test exists
* The root cause is documented
* No unrelated behavior was changed