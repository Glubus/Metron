---
trigger: always_on
---

# Input Validation & Trust Boundary Rule

## Concise Lesson

All external inputs are **untrusted by default**. Validation must occur at system boundaries before any processing.

---

## Core Principles

1. **Trust Boundary**:

   * Any data coming from APIs, users, files, network, S3, env vars, or other services is considered **hostile**.
   * Validation must happen **before** business logic.

2. **Fail Fast**:

   * Invalid input must be rejected early with a clear, domain-specific error.
   * Never let invalid data propagate deeper into the system.

3. **Explicit Limits**:
   Every external input must define:

   * maximum size (bytes)
   * allowed formats / values
   * valid ranges

4. **No Implicit Assumptions**:

   * Never assume non-zero sizes, valid formats, or sane defaults.
   * Zero, empty, oversized, and malformed inputs must be handled explicitly.

---

## Required Validations (Examples)

### Media / Image Processing

* Maximum file size (e.g., 10 MB)
* Maximum dimensions (e.g., 8k x 8k)
* Allowed formats only (JPEG, PNG, WebP)
* Width / height must be > 0

### Network / Storage

* Timeouts must be defined
* Retries must be bounded
* Object existence must be verified

---

## Good Example

```rust
fn validate_resize_request(width: u32, image_size: usize) -> Result<(), ResizeError> {
    if width == 0 {
        return Err(ResizeError::InvalidDimension(width));
    }
    if image_size > MAX_IMAGE_BYTES {
        return Err(ResizeError::ImageTooLarge(image_size));
    }
    Ok(())
}
```

---

## Bad Example

```rust
// BAD: Assumes input is always valid
let img = image::load_from_memory(&bytes).unwrap();
```

---

## Intent

* Prevent security issues and denial-of-service risks
* Protect CPU and memory budgets
* Make failure modes explicit and predictable
* Ensure system robustness under hostile conditions
