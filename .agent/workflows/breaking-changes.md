---
description: Breaking Change Workflow
---

## Purpose

Introduce breaking changes **safely**, **predictably**, and **without surprises**.

This workflow applies to:
- public APIs
- shared libraries
- long-lived internal interfaces

---

## Principles

- Breaking changes are intentional, never accidental
- Users must have time to adapt
- Every breaking change must be documented

---

## Steps

### 1. Formalize the Decision

Before making the change:

- Record a decision in `wiki.wiki`
- Explicitly state:
  - what is changing
  - why it must break compatibility
  - why alternatives were rejected

> If the decision is not documented, the change must not proceed.

---

### 2. Announce the Deprecation

If possible:

- Mark the old behavior as deprecated
- Add:
  - warnings
  - logs
  - documentation notes

Example:
```rust
#[deprecated(note = "Use resize_with_policy instead")]
pub fn resize_legacy(...) { ... }
````

---

### 3. Provide a Migration Path

Users must be able to migrate easily:

* Provide a replacement API
* Include usage examples
* Keep semantics clear and consistent

> Never remove functionality without a clear alternative.

---

### 4. Update Documentation and Changelog

Before release:

* Update `changelog.md`
* Clearly mark the change as **BREAKING**
* Reference the wiki decision

Example:

```markdown
### Changed
- **BREAKING**: Removed legacy PNG output. Use WebP only.
```

---

### 5. Delay Removal

Rules:

* Deprecation and removal must not happen in the same release
* Allow at least one release cycle for migration

---

### 6. Remove and Clean Up

After the deprecation period:

* Remove old code paths
* Remove compatibility layers
* Clean up related tests and docs

---

## Exit Criteria

A breaking change is complete only if:

* The decision is documented
* A migration path exists
* Users were warned in advance
* Changelog clearly marks the break


