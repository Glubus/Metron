---
description: Feature Development Workflow (Prod-Grade)
---

## Purpose

Deliver a new feature with:
- zero hidden technical debt
- full traceability
- test coverage
- production readiness

This workflow must be followed for **any non-trivial feature**.

---

## Steps

### 1. Clarify the Intent

Before writing code:

- Write or update a decision entry in `wiki.wiki`
- Answer explicitly:
  - What problem are we solving?
  - Why now?
  - Why this approach?
  - Load Any Rules or Skills needed for resolving this matter

> If the intent is not clear, do not start coding.

---

### 2. Create a Dedicated Branch

Never work directly on `main` or `dev`.

```bash
git checkout -b feature/US123-short-feature-description
````

Branch name must reflect:

* the user story / ticket
* the feature scope

---

### 3. TDD Loop (Mandatory)

Follow strict Test Driven Development:

1. Write a failing test
2. Implement the minimal code to pass
3. Refactor if needed
4. Repeat

Rules:

* No production code without a test
* Tests must be deterministic
* Tests must reflect real use cases

---

### 4. Update Changelog

At the end of the TDD cycle:

* Update `changelog.md`
* Include:

  * what was added/changed
  * technical details (libs, algorithms, performance impact)

---

### 5. Add Observability

For any meaningful feature:

* Add structured logs on critical paths
* Add metrics if the feature:

  * is CPU intensive
  * is IO heavy
  * is on a hot path

---

### 6. Commit Changes

Use Conventional Commits:

```bash
git commit -m "feat: US123-add-image-resize-cache"
```

Rules:

* One logical feature per commit
* No mixed refactors + features

---

### 7. Review Checklist (Self-Review)

Before merging, verify:

* [ ] Naming conventions respected
* [ ] No blocking code in async context
* [ ] Errors are domain-specific
* [ ] Wiki updated if a decision was made
* [ ] Feature is deterministic and cacheable if applicable

---

## Exit Criteria

A feature is considered **done** only if:

* Tests are green
* Changelog is updated
* Wiki reflects architectural decisions
* Code follows all project rules