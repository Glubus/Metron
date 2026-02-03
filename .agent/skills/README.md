# Rust Agent Skills

This directory contains specialized skills that extend the agent's capabilities for Rust development in the Rhythm Open Exchange project.

## Available Skills

### 1. **rust-error-handling** 

[📄 View Skill](rust-error-handling/SKILL.md)

**When to use**: Creating error types, handling Results, implementing error propagation.

**Key principles**:

- Zero `unwrap()` / `expect()` in production code
- Domain-specific errors with `thiserror`
- Avoid `anyhow` in library code
- Provide context in error messages

**Covers**:

- Error type design patterns
- Result chaining and propagation
- Custom error conversions
- Testing error paths
- Integration with observability

---

### 2. **rust-async-patterns**

[📄 View Skill](rust-async-patterns/SKILL.md)

**When to use**: Implementing async functions, handling blocking operations, managing concurrent tasks.

**Key principles**:

- Never block the async runtime
- Always set timeouts on I/O
- Bound concurrent operations
- Use structured concurrency

**Covers**:

- `spawn_blocking` for CPU-bound work
- Timeout and retry patterns
- Concurrent stream processing
- Graceful shutdown and cancellation
- Resource budgets and backpressure

---

### 3. **rust-ffi-bindings**

[📄 View Skill](rust-ffi-bindings/SKILL.md)

**When to use**: Creating C APIs, C# bindings, Python bindings, or WebAssembly exports.

**Key principles**:

- Safety at the boundary (panic handling)
- ABI stability with `#[repr(C)]`
- Explicit memory ownership
- Error codes, never panics

**Covers**:

- C API patterns with opaque handles
- C# / .NET P/Invoke bindings
- Python bindings with PyO3
- WebAssembly with wasm-bindgen
- Memory management across FFI
- Buffer management patterns

---

### 4. **rust-performance-optimization**
[📄 View Skill](rust-performance-optimization/SKILL.md)

**When to use**: Optimizing hot paths, reducing allocations, improving cache locality, benchmarking.

**Key principles**:
- Profile before optimizing
- Minimize allocations
- Use `Copy` types for small data
- Optimize data layout for cache

**Covers**:
- Profiling workflow with criterion and flamegraph
- Buffer reuse and allocation strategies
- SIMD for bulk operations
- Compile-time optimization (LTO, const eval)
- Memory layout optimization
- Benchmarking best practices

---

### 5. **codec-development**
[📄 View Skill](codec-development/SKILL.md)

**When to use**: Adding new format support, implementing parsers, working with format conversion.

**Key principles**:
- Validate all input
- Fail fast with clear errors
- Preserve format fidelity
- Handle format-specific quirks

**Covers**:
- Decoder patterns (text and binary)
- Encoder patterns with size estimation
- Roundtrip testing
- Fuzzing for robustness
- Format-specific quirk handling
- Input validation strategies

---

## Skill Structure

Each skill follows this structure:

```
.agent/skills/
└── {skill-name}/
    └── SKILL.md          # Main instructions (required)
```

Future skills may include additional resources:
- `scripts/` - Helper scripts
- `examples/` - Reference implementations
- `resources/` - Templates and assets

## How Skills Work

1. **Discovery**: When a conversation starts, the agent sees all available skills with their descriptions
2. **Activation**: If a skill looks relevant, the agent reads the full `SKILL.md` content
3. **Execution**: The agent follows the skill's instructions while working on your task

You don't need to explicitly invoke skills—the agent decides based on context. However, you can mention a skill by name to ensure it's used.

## Creating New Skills

To create a new skill:

1. Create a folder: `.agent/skills/{skill-name}/`
2. Add `SKILL.md` with YAML frontmatter:

```markdown
---
name: my-skill
description: Clear description of what the skill does and when to use it.
---

# My Skill

Detailed instructions for the agent...
```

### Frontmatter Fields

- **name** (optional): Unique identifier (defaults to folder name)
- **description** (required): What the skill does and when to use it

## Best Practices

### Keep Skills Focused
Each skill should do one thing well. Create separate skills for distinct tasks.

### Write Clear Descriptions
The description is how the agent decides whether to use your skill. Make it specific about what the skill does and when it's useful.

### Include Decision Trees
For complex skills, add sections that help the agent choose the right approach based on the situation.

### Provide Examples
Show both good and bad examples to illustrate patterns clearly.

## Integration with Project Rules

These skills are designed to work seamlessly with the project's strict rules:

- **rule-derministic.md** - Idempotency & determinism
- **rule-doc.md** - Documentation standards
- **rule-git.md** - Git workflow & changelog
- **rule-input.md** - Input validation & trust boundary
- **rule-memory-and-ressources.md** - Resource & memory budgets
- **rule-naming.md** - Naming conventions
- **rule-observability.md** - Logging, metrics, traces
- **rule-testing-pyramid.md** - Testing strategy
- **rust-strict-standards.md** - Core Rust standards

Skills reference these rules and provide practical implementation guidance.

## References

- [Google ADK Skills Documentation](https://developers.google.com/adk/skills)
- [Antigravity Agent Documentation](https://developers.google.com/antigravity)
- Project Wiki: `.wiki/`
- Project Rules: `.agent/rules/`

## Contributing

When adding new skills:

1. Follow the skill structure outlined above
2. Ensure the description clearly indicates when to use the skill
3. Include practical examples and decision trees
4. Reference relevant project rules
5. Add the skill to this README

---

**Last Updated**: 2026-01-30

**Skills Count**: 5

**Maintained by**: ROX Development Team
