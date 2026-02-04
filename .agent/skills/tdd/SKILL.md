---
name: tdd
description: A comprehensive guide to Test Driven Development (TDD), emphasizing the Red-Green-Refactor cycle and universal testing principles usable in any project.
---

# Test Driven Development (TDD) Skill

This skill defines the workflow and mindset for Test Driven Development. It is designed to be **universal**: apply these principles regardless of the language (Rust, TS, Python) or framework.

## 1. The Core Mantra: Red - Green - Refactor

You must never write production code unless you have a failing test that demands it.

### 🔴 Red: Write a Failing Test

1.  **Understand the requirement**: What specific behavior are you implementing?
2.  **Write the test**: Call the function/API as you *wish* it existed.
3.  **Run the test**: It **must fail**.
    *   *Compilation Error* counts as a failure (e.g., function doesn't exist).
    *   *Assertion Error* counts as a failure (e.g., returns wrong value).
    *   *Assertion Count* must be 1 or 2 at maximum no more one test for one behavior.
4.  **Confirm the failure**: Verify it failed for the *expected reason*.

### 🟢 Green: Make it Pass

1.  **Do the simplest thing**: Write the minimum amount of code to make the test pass.
2.  **Sin is allowed**: Hardcoding values, copy-pasting, and "ugly" code are acceptable here to get to Green quickly.
3.  **Run the test**: It must pass.

### 🔵 Refactor: Clean it Up
1.  **Improve code quality**: Remove duplication, rename variables, extract functions.
2.  **Preserve behavior**: Run tests after *every* small change.
3.  **Optimize**: Improve performance if needed, but safe behind the wall of passing tests.

## 2. Breaking It Down: ZOMBIES

When you don't know what test to write next, use ZOMBIES:

*   **Z** - **Zero**: Test the empty case (empty string, empty list, zero input).
*   **O** - **One**: Test the single element case.
*   **M** - **Many**: Test multiple elements, complex inputs.
*   **B** - **Boundary**: Test edge cases (max int, null, undefined, timeouts).
*   **I** - **Interface**: Test the API signature and contract.
*   **E** - **Exceptions**: Test error handling and failure modes.
*   **S** - **Simple**: Test the simplest "happy path" first.

## 3. The Testing Pyramid

Structure your test suite to balance confidence and speed.

### 1. Unit Tests (The Base - 70%)
*   **Scope**: Isolated functions, classes, or modules.
*   **Speed**: Extremely fast (<1ms).
*   **Deps**: No external dependencies (no DB, no FileSystem, no Network). Mock everything else.
*   **When**: Written for every logic branch.

### 2. Integration Tests (The Middle - 20%)
*   **Scope**: Interaction between modules or with external systems (DB, API).
*   **Speed**: Slower (ms to seconds).
*   **Deps**: Use real or containerized dependencies (e.g., Testcontainers, in-memory DBs).
*   **When**: To verify that components talk to each other correctly.

### 3. End-to-End (E2E) Tests (The Peak - 10%)
*   **Scope**: The entire system from user's perspective.
*   **Speed**: Slow (seconds to minutes).
*   **Deps**: Full production-like environment.
*   **When**: Critical user flows (Smoke Tests).

## 4. Generalization: How to Use This Skill

This skill is a *meta-skill*. When assigned a task, apply TDD as follows:

1.  **Identify the framework**: Is it `cargo test` (Rust), `jest` (TS), `pytest` (Python)?
2.  **Locate the test file**: create `tests/` or `__tests__` or `mod tests`.
3.  **Iterate**:
    *   *Agent*: "I need to implement feature X."
    *   *TDD Check*: "Do I have a test for X?" -> No.
    *   *Action*: Write test for X. Run it. Fail.
    *   *Action*: Implement X. Run test. Pass.
    *   *Action*: Refactor.

## 5. Checkpoints for Review

When reviewing code or asking for review, verify:
- [ ] **Test Exists**: Does every new feature have a corelating test?
- [ ] **Test Fails**: Did we see it fail? (Prevent false positives).
- [ ] **Test is Clean**: Is the test readable? (Tests are documentation).
- [ ] **No Flakiness**: Is the test deterministic?

---
*Reference this skill whenever you are about to write logic.*
