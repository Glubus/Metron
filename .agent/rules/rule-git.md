---
trigger: always_on
---

# Git Workflow & Changelog Rule

## Concise Lesson

Strict and auditable version-control discipline to ensure traceability, safety, and long-term maintainability.


---

## 1. Branching Rules

1. **Protected Branches**:

   * **NEVER** work directly on `main` or `dev`.
   * All work must be done in a dedicated branch.

2. **Branch Naming**:

   * `feature/<short-description>` (e.g., `feature/webp-support`)
   * `fix/<short-description>` (e.g., `fix/memory-leak`)
   * `chore/<short-description>` (e.g., `chore/deps-update`)

3. **Exceptions**:

   * Direct commits to `main` or `dev` are allowed **only if the user explicitly orders it**.
   * Emergency hotfixes must still be documented in the changelog.

4. **Merge Policy**:

   * Prefer **rebase or squash merge** to keep history readable.
   * Never force-push shared branches (`main`, `dev`).

---

## 2. TDD & Changelog Discipline

1. **TDD Cycle Completion**:

   * At the end of **every completed TDD cycle**, the `changelog.md` **must** be updated.
   * Incomplete or failing tests must not be logged.

2. **Changelog Content**:

   * Human-readable summary of changes.
   * Technical details when relevant:

     * libraries added or removed
     * algorithmic or architectural changes
     * performance or memory impact

3. **Scope**:

   * Log functional changes, behavior changes, and technical decisions.
   * Do **not** log mechanical refactors or formatting-only changes.

---
## 4. Continuous Release Policy

1. **Micro-Releases**:
   * For every completed small feature or fix, **bump the `patch` version** (e.g., 0.5.0 -> 0.5.1) in `Cargo.toml`.
   * **Commit the version bump** along with the feature code and changelog update.

---

## 3. Commit Message Rules

1. **Convention**:

   * Follow **Conventional Commits**:

     ```text
     <type>: <subject>
     ```

2. **Allowed Types**:

   * `feat` → new feature
   * `fix` → bug fix
   * `chore` → tooling, dependencies, cleanup
   * `test` → tests only
   * `docs` → documentation only

3. **Traceability (US)**:

   * If a User Story or ticket exists, prepend it:

     ```text
     feat: US123-add-webp-resizing
     ```
   * If the US number is unknown, the commit **must not be created** and the user must be asked.

4. **Quality Bar**:

   * No generic messages (`update`, `wip`, `fix stuff`).
   * The subject must describe **what changed**, not how.

---

## Good Example

**Scenario**: Finished implementing WebP resizing.

1. **Branch**:

   ```text
   feature/webp-support
   ```

2. **Changelog Update**:

   ```markdown
   ## [Unreleased] - 2023-10-27
   ### Added
   - WebP support in `ResizeManager`.
   - `image` crate with `webp` feature enabled.

   ### Performance
   - ~40% bandwidth reduction compared to PNG outputs.
   ```

3. **Commit**:

   ```bash
   git commit -m "feat: US123-add-webp-resizing"
   ```

---

## Bad Example

**Scenario**: "I’ll just fix this quickly on main."

1. **Branch**:

   ```text
   main   # BAD: protected branch
   ```

2. **Commit**:

   ```bash
   git commit -m "update"   # BAD: non-descriptive
   ```

3. **Changelog**:

   * Not updated ❌

---

## Intent

* Git history must be **readable by humans and machines**.
* Changelog acts as the **user-facing and technical evolution log**.
* Branching + commits + changelog together form a single traceable unit of change.