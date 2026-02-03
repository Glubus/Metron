---
trigger: always_on
---

# Wiki Workflow Rule

## Concise Lesson

The Wiki (`wiki.wiki/` or `wiki.md`) is the long-term brain of the project:

1. **Logging**: Every major feature or technical decision must be logged (A decision is “major” if reverting it would impact multiple files, APIs, or users.).
2. **Traceability**: "Why did we do this?" must find its answer in the wiki.
3. **Update**: The wiki must be updated *at the same time* the code is committed.
4. **Structure**: Maintain clear sections (Decisions, Roadmap, Changelog).

## Good Example

*Excerpt from `wiki.wiki`*:

```markdown
### 2023-10-27: Adoption of WebP
**Decision**: Forced switch to WebP for all API outputs.
**Why**: Load tests show 40% bandwidth reduction vs PNG.
**Impact**: Requires `image` crate with `webp` feature.
```

## Bad Example

*Empty or obsolete Wiki.*
"We changed the DB last week but I forgot to note it down."
-> Loss of contextual knowledge.