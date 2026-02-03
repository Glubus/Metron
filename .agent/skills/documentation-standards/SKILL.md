---
name: documentation-standards
description: Enforces strict checkpoints and formats for Wiki, Rust Docs, Changelog, and Readme to ensure long-term maintainability and traceability.
---

# Documentation Standards Skill

This skill defines the authoritative standards for all documentation in the project. You must follow these checkpoints when writing or updating documentation.

## 1. Wiki (`wiki.wiki` or `docs/wiki`)

The Wiki is the "long-term brain" of the project. It stores architectural decisions, rationale, and knowledge that must survive code refactors.

### Checkpoints
- [ ] **Traceability**: Every major technical decision (one that is hard to reverse) has an entry.
- [ ] **Format**: Each entry follows the "Decision Log" format.
- [ ] **Timing**: Wiki is updated *in the same PR* as the code change.
- [ ] **Location**: Stored in `wiki.wiki/` (submodule) or `docs/wiki/`.

### Format: Decision Log (ADR-lite)

```markdown
### YYYY-MM-DD: [Title of Decision]
**Decision**: [What did we decide? e.g., "Adopted `thiserror` for library errors."]
**Why**: [Rationale. e.g., "Provides better ergonomics than `anyhow` for library consumers and avoids dynamic dispatch overhead."]
**Impact**: [Consequences. e.g., "Requires refactoring `src/error.rs` and updating dependencies."]
**Status**: [Accepted | Proposed | Deprecated]
```

## 2. Rust Documentation (`///`)

Documentation is code. It must be written *during* development, not after.

### Checkpoints
- [ ] **Public API**: All `pub` functions, structs, enums, and traits must have `///` doc comments.
- [ ] **The "Why"**: Docs explain *why* this approach was taken, not just what the code does.
- [ ] **Sections**: Use standard sections (`# Returns`, `# Errors`, `# Panics`, `# Safety`).
- [ ] **Examples**: Include at least one doctest example for complex functions.

### Format: Standard Rust Doc

```rust
/// [Short summary line].
///
/// [More detailed explanation of the function's purpose and behavior].
///
/// # Why
/// [Explain the rationale for the implementation choice, algorithm, or library used].
///
/// # Arguments
/// * `arg_name` - [Description of argument]
///
/// # Returns
/// [Description of return value]
///
/// # Errors
/// [List of error conditions and variants returned]
///
/// # Panics
/// [Conditions that cause a panic, if any]
///
/// # Safety
/// [Required if function is `unsafe`. List invariants the caller must uphold.]
pub fn my_function() {}
```

## 3. Changelog (`changelog.md`)

The changelog is the user-facing history of the project. It must be readable by humans.

### Checkpoints
- [ ] **TDD Cycle**: Updated at the end of *every* completed TDD cycle.
- [ ] **Unreleased**: Changes go under `## [Unreleased]` first.
- [ ] **Format**: Keep a "Keep a Changelog" format.
- [ ] **Content**: Focus on *value* (features, fixes), not just valid commits. Technical details are allowed if impactful.

### Format: Keep a Changelog

```markdown
## [Unreleased] - YYYY-MM-DD

### Added
- Feature X description.
- [Tech] Added `dep-name` for X.

### Changed
- Refactored Y to improve performance by Z%.

### Fixed
- Crash when input is empty (fixes #123).
```

## 4. Readme (`README.md`)

The entry point for the project. It must be concise and actionable.

### Checkpoints
- [ ] **First Impression**: Project title and 1-sentence value proposition at the top.
- [ ] **Quick Start**: "How to run" is visible without scrolling far.
- [ ] **Prerequisites**: Clear list of tools needed (Rust, Cargo, external libs).
- [ ] **Status**: Badges (Build, Test, Version) if available.

### Format: Standard Readme

```markdown
# [Project Name]

[Short, punchy description of what the project does].

## 🚀 Quick Start

\`\`\`bash
# Install
cargo install --path .

# Run
my-app --help
\`\`\`

## 🛠 Prerequisites

- Rust 1.75+
- [External Tool if needed]

## 🏗 Architecture

See [wiki.wiki](wiki.wiki) for detailed architectural decisions.
```
