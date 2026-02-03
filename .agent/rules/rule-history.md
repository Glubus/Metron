---
trigger: always_on
---

# History Organization Rule

## Concise Lesson

To maintain a clean, machine-fragmented, and safe history system:

1. **Structure**: All chat histories, raw contexts, or session traces must be stored under the `.history/` directory.
2. **Sub-Directory**: Each machine or environment must have its own sub-directory named by its hostname or unique identifier (e.g., `.history/{hostname}/`).
3. **Isolation**: Never mix logs or sessions from different machines at the root of `.history/`.
4. **Versioning Policy**: The `.history/` directory is **local-only by default** and must not be committed to the main repository unless explicitly stated.
5. **Safety**: Never store secrets, credentials, tokens, or sensitive personal data inside `.history/`.
6. **File Naming**: Use explicit, sortable filenames with ISO-8601 dates and intent prefixes (e.g., `session_2023-10-27.md`, `context_init.md`, `deploy_2023-11-02.md`).

## Good Example

File structure:

```bash
.history/
├── my-laptop/
│   ├── context_init.md
│   ├── session_2023-10-27.md
│   └── session_2023-10-28.md
├── prod-server/
│   └── deploy_2023-11-02.md
```

## Bad Example

Flat and disorganized structure:

```bash
.history/
├── chat_log.md            # BAD: Stored at root
├── session_old.md         # BAD: No machine isolation
├── macbook_pro_session.md # BAD: Filename embeds machine name
```

## Intent

* `.history/` represents **short-term, raw, machine-scoped memory**.
* It complements (but does not replace) the **Wiki**, which stores long-term decisions and rationale.
* This separation prevents context pollution, improves traceability, and makes automation reliable.
