---
trigger: always_on
---

# Observability Rule

## Concise Lesson

A system that cannot be observed cannot be operated. Logs, metrics, and traces are mandatory for any non-trivial system.

---

## Core Principles

1. **Structured Logging**:

   * Logs must be structured (key-value), not free text.
   * Logs must include contextual identifiers (request_id, correlation_id).

2. **Errors Are Signals**:

   * Every error path must emit a log entry.
   * Silent failures are forbidden.

3. **Metrics Over Logs**:

   * Anything you want to alert on must be a metric, not a log.
   * Logs explain *why*, metrics show *what*.

4. **Tracing for Boundaries**:

   * Cross-boundary operations (API → S3 → CPU task) must be traceable.
   * Async and blocking boundaries must be visible.

---

## Required Signals

### Logs

* Request start / end
* Error conditions with context
* Resource limit rejections

### Metrics

* Request count and error rate
* Resize duration (CPU)
* Image size distribution
* spawn_blocking queue saturation (if applicable)

### Traces

* API request lifecycle
* External calls (S3, network)
* CPU-bound tasks

---

## Good Example

```rust
use tracing::{info, error, instrument};

#[instrument(skip(image_data))]
pub async fn process_image_for_web(
    image_data: Vec<u8>,
    target_width: u32,
) -> Result<Vec<u8>, ResizeError> {
    info!(target_width, "starting image resize");

    let result = tokio::task::spawn_blocking(move || resize(image_data, target_width))
        .await
        .map_err(|e| {
            error!(error = %e, "blocking task failed");
            ResizeError::Internal
        })?;

    Ok(result)
}
```

---

## Bad Example

```rust
// BAD: No logs, no metrics, no trace context
pub async fn resize(bytes: Vec<u8>, width: u32) -> Vec<u8> {
    resize_sync(bytes, width)
}
```

---

## Intent

* Enable production debugging and incident response
* Provide visibility into performance and failures
* Support alerting and capacity planning
* Make async and blocking behavior explicit
