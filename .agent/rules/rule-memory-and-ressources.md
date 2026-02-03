---
trigger: always_on
---

# Resource & Memory Budget Rule

## Concise Lesson

CPU, memory, and I/O are finite resources. Any operation that consumes them significantly must declare explicit limits and failure modes.

---

## Core Principles

1. **Explicit Budgets**:

   * Any CPU- or memory-intensive operation must define:

     * maximum input size
     * memory budget
     * execution model (async vs blocking)

2. **No Unbounded Work**:

   * Never process unbounded input in memory.
   * Never spawn unbounded blocking tasks.

3. **Backpressure Awareness**:

   * The system must degrade gracefully under load.
   * Prefer rejecting work over exhausting resources.

4. **Fail Predictably**:

   * Resource exhaustion must result in controlled, domain-specific errors.
   * OOM or executor starvation is considered a bug.

---

## Required Declarations (Examples)

### Image Processing

* Maximum image bytes in memory
* Maximum image dimensions
* Maximum resize target width
* CPU-bound work must run in `spawn_blocking`

### Async Systems

* Maximum concurrent blocking tasks
* Timeouts on I/O operations
* Bounded channels and queues

---

## Good Example

```rust
const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024; // 10 MB
const MAX_TARGET_WIDTH: u32 = 4096;

async fn resize_image_safe(bytes: Vec<u8>, width: u32) -> Result<Vec<u8>, ResizeError> {
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(ResizeError::ImageTooLarge(bytes.len()));
    }
    if width > MAX_TARGET_WIDTH {
        return Err(ResizeError::InvalidDimension(width));
    }

    tokio::task::spawn_blocking(move || resize(bytes, width)).await?
}
```

---

## Bad Example

```rust
// BAD: Unbounded memory and CPU usage
pub async fn resize(bytes: Vec<u8>, width: u32) -> Vec<u8> {
    image::load_from_memory(&bytes).unwrap();
    // heavy CPU work on async thread
}
```

---

## Intent

* Prevent denial-of-service and resource exhaustion
* Make system limits explicit and reviewable
* Enable safe scaling under load
* Protect async runtimes from starvation
