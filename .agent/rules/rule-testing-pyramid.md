---
trigger: always_on
---

# Testing Pyramid Rule

## Concise Lesson

Tests must be structured to maximize confidence while minimizing maintenance cost. Not all tests have the same value.

---

## Core Principles

1. **Pyramid Structure**:

   * Many **unit tests** (fast, deterministic)
   * Fewer **integration tests** (real components, mocked boundaries)
   * Very few **end-to-end tests** (only critical paths)

2. **Deterministic First**:

   * Tests must be deterministic and repeatable.
   * Flaky tests are considered broken tests.

3. **Boundaries Matter**:

   * External systems (S3, network, filesystem) must be mocked or isolated.
   * Integration tests may use controlled environments (e.g., MinIO).

4. **TDD Alignment**:

   * Unit tests are written **before** implementation.
   * Integration tests are added once behavior is stable.

5. **Single Assertion Principle**:

   * **One Test = One Concept = One Assertion**.
   * Avoid multiple assertions in a single test function.
   * If a test fails, the cause must be unambiguous.

---

## Test Types

### Unit Tests

* Pure functions
* No I/O
* No async runtime required
* Run on every commit

### Integration Tests

* Real components interacting together
* Mocked or local dependencies (MinIO, temp dirs)
* Validate contracts and boundaries

### End-to-End Tests

* Validate user-visible behavior only
* Slow and expensive
* Must justify their existence

---

## Good Example

```rust
#[test]
fn calculate_dimensions_keeps_ratio() {
    let (w, h) = calculate_dimensions(400, 200, 100).unwrap();
    assert_eq!((w, h), (100, 50));
}
```

```rust
#[tokio::test]
async fn resize_with_minio_bucket() {
    let bucket = setup_test_bucket();
    let resizer = S3Resizer::new(bucket);
    let result = resizer.get_resized_image("test.jpg", 100).await;
    assert!(result.is_ok());
}
```

---

## Bad Example

```rust
// BAD: Only end-to-end tests, slow and flaky
#[tokio::test]
async fn full_system_test() {
    call_real_s3();
    resize_large_image();
}
```

---

## Intent

* Keep feedback loops fast
* Reduce test maintenance cost
* Increase confidence in core logic
* Prevent fragile test suites
