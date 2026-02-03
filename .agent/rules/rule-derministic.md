---
trigger: always_on
---

# Idempotency & Determinism Rule

## Concise Lesson

Given the same input, the system must always produce the same output. Repeating an operation must not change the result or create unintended side effects.

---

## Core Principles

1. **Deterministic Behavior**:

   * Pure transformations must be deterministic.
   * No hidden randomness, time-based behavior, or environment-dependent output.

2. **Idempotent Operations**:

   * Repeating the same request must yield the same result.
   * Safe retries must not corrupt state or produce duplicates.

3. **Stable Inputs → Stable Outputs**:

   * Output must depend only on explicit inputs.
   * Configuration and policies must be versioned.

4. **Retry Safety**:

   * Any operation that may be retried must be designed for idempotency.

---

## Common Sources of Non-Determinism

* Random number generators without fixed seeds
* Timestamps embedded in outputs
* Environment-dependent behavior
* Unordered data structures (e.g., HashMap iteration)

---

## Good Example

```rust
fn resize_image_deterministic(
    image: DynamicImage,
    width: u32,
) -> DynamicImage {
    image.resize(width, image.height(), image::imageops::FilterType::Lanczos3)
}
```

```text
Cache key = hash(path + width + filter + format)
```

---

## Bad Example

```rust
// BAD: Output depends on current time
let filename = format!("resized_{}.webp", chrono::Utc::now());
```

---

## Intent

* Enable safe retries and caching
* Support CDN and distributed systems
* Make behavior predictable and testable
* Reduce production surprises
