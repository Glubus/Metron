---
name: rust-async-patterns
description: Async/await patterns and tokio best practices for Rust. Use when implementing async functions, handling blocking operations, managing concurrent tasks, or working with async runtimes. Enforces proper resource budgets and backpressure handling.
---

# Rust Async Patterns Skill

This skill provides guidance on writing correct, efficient, and safe asynchronous Rust code using tokio and async/await.

## When to use this skill

- Implementing async functions or services
- Converting blocking code to async
- Managing concurrent operations
- Working with tokio runtime
- Handling I/O-bound or CPU-bound work in async context
- Implementing timeouts and cancellation

## Core Principles

### 1. Never Block the Async Runtime

**Rule**: CPU-intensive or blocking operations must run in `spawn_blocking`, never directly in async functions.

**Why**: Blocking the async executor starves other tasks and degrades performance.

**Bad**:

```rust
// BAD: Blocks the async runtime
pub async fn process_image(data: Vec<u8>) -> Result<Vec<u8>, Error> {
    // Heavy CPU work on async thread!
    let img = image::load_from_memory(&data)?;
    let resized = img.resize(100, 100, FilterType::Lanczos3);
    Ok(resized.to_bytes())
}
```

**Good**:

```rust
// GOOD: CPU work in blocking thread pool
pub async fn process_image(data: Vec<u8>) -> Result<Vec<u8>, Error> {
    tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&data)?;
        let resized = img.resize(100, 100, FilterType::Lanczos3);
        Ok(resized.to_bytes())
    })
    .await
    .map_err(|e| Error::JoinError(e))?
}
```

### 2. Always Set Timeouts on I/O Operations

**Rule**: All network I/O, file I/O, and external calls must have explicit timeouts.

**Why**: Prevents indefinite hangs and resource exhaustion.

```rust
use tokio::time::{timeout, Duration};

pub async fn fetch_data(url: &str) -> Result<String, Error> {
    let request = reqwest::get(url);
    
    match timeout(Duration::from_secs(10), request).await {
        Ok(Ok(response)) => Ok(response.text().await?),
        Ok(Err(e)) => Err(Error::RequestFailed(e)),
        Err(_) => Err(Error::Timeout),
    }
}
```

### 3. Bound Concurrent Operations

**Rule**: Never spawn unbounded concurrent tasks. Use semaphores or buffered channels.

**Why**: Prevents resource exhaustion and OOM conditions.

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

const MAX_CONCURRENT: usize = 10;

pub async fn process_batch(items: Vec<Item>) -> Result<Vec<Output>, Error> {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
    let mut tasks = Vec::new();
    
    for item in items {
        let permit = semaphore.clone().acquire_owned().await?;
        
        let task = tokio::spawn(async move {
            let result = process_item(item).await;
            drop(permit); // Release permit
            result
        });
        
        tasks.push(task);
    }
    
    // Collect results
    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await??);
    }
    
    Ok(results)
}
```

### 4. Use Structured Concurrency

**Rule**: Prefer `tokio::select!`, `join!`, and `try_join!` over manual task spawning when possible.

**Why**: Ensures proper cancellation and resource cleanup.

```rust
use tokio::try_join;

pub async fn load_chart_with_audio(
    chart_path: &Path,
    audio_path: &Path,
) -> Result<(RoxChart, AudioData), Error> {
    // Both load concurrently, both must succeed
    let (chart, audio) = try_join!(
        load_chart(chart_path),
        load_audio(audio_path)
    )?;
    
    Ok((chart, audio))
}
```

## Common Async Patterns

### Pattern 1: Async Function with Timeout

```rust
use tokio::time::{timeout, Duration};

pub async fn decode_with_timeout(
    data: &[u8],
    max_duration: Duration,
) -> Result<RoxChart, CodecError> {
    timeout(max_duration, async {
        // Actual decode work
        decode_chart(data)
    })
    .await
    .map_err(|_| CodecError::Timeout)?
}
```

### Pattern 2: CPU-Bound Work in Async Context

```rust
pub async fn compute_hash(data: Vec<u8>) -> Result<Hash, Error> {
    tokio::task::spawn_blocking(move || {
        // CPU-intensive hashing
        blake3::hash(&data)
    })
    .await
    .map_err(|e| Error::TaskFailed(e.to_string()))
}
```

### Pattern 3: Concurrent Stream Processing

```rust
use futures::stream::{self, StreamExt};

pub async fn process_charts_concurrent(
    paths: Vec<PathBuf>,
    concurrency: usize,
) -> Vec<Result<RoxChart, CodecError>> {
    stream::iter(paths)
        .map(|path| async move {
            decode_chart(&path).await
        })
        .buffer_unordered(concurrency)
        .collect()
        .await
}
```

### Pattern 4: Graceful Shutdown with Cancellation

```rust
use tokio::sync::watch;
use tokio::select;

pub async fn worker_with_shutdown(
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), Error> {
    loop {
        select! {
            _ = shutdown_rx.changed() => {
                info!("Shutdown signal received");
                break;
            }
            result = do_work() => {
                result?;
            }
        }
    }
    
    Ok(())
}
```

### Pattern 5: Retry with Exponential Backoff

```rust
use tokio::time::{sleep, Duration};

pub async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_retries: u32,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut attempt = 0;
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= max_retries => return Err(e),
            Err(_) => {
                attempt += 1;
                let delay = Duration::from_millis(100 * 2_u64.pow(attempt));
                sleep(delay).await;
            }
        }
    }
}
```

## Decision Tree

### When to use async vs sync?

1. **Is this I/O-bound work?** (network, file, database)
   - Yes → Use async
   - No → Continue to next question

2. **Is this CPU-bound work?**
   - Yes → Use `spawn_blocking` if called from async context
   - No → Continue to next question

3. **Do I need concurrency?**
   - Yes → Use async
   - No → Sync is simpler

### When to use `spawn` vs `spawn_blocking`?

1. **Is the work CPU-intensive?** (image processing, hashing, compression)
   - Yes → Use `spawn_blocking`
   - No → Continue

2. **Does it call blocking APIs?** (std::fs, blocking network)
   - Yes → Use `spawn_blocking`
   - No → Use `spawn` for async work

### How to handle multiple concurrent operations?

1. **Must all succeed?**
   - Yes → Use `try_join!`
   - No → Continue

2. **Do I need first result?**
   - Yes → Use `select!`
   - No → Use `join!` or manual task collection

## Resource Budgets

Always define explicit limits:

```rust
// Configuration for async operations
pub struct AsyncConfig {
    /// Maximum concurrent blocking tasks
    pub max_blocking_tasks: usize,
    
    /// Maximum concurrent I/O operations
    pub max_io_concurrent: usize,
    
    /// Default timeout for I/O operations
    pub io_timeout: Duration,
    
    /// Maximum retry attempts
    pub max_retries: u32,
}

impl Default for AsyncConfig {
    fn default() -> Self {
        Self {
            max_blocking_tasks: num_cpus::get(),
            max_io_concurrent: 100,
            io_timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}
```

## Common Mistakes to Avoid

### ❌ Blocking in async functions

```rust
// BAD
pub async fn process(data: Vec<u8>) -> Result<Vec<u8>, Error> {
    std::thread::sleep(Duration::from_secs(1)); // Blocks executor!
    Ok(data)
}
```

```rust
// GOOD
pub async fn process(data: Vec<u8>) -> Result<Vec<u8>, Error> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(data)
}
```

### ❌ No timeout on I/O

```rust
// BAD
pub async fn fetch(url: &str) -> Result<String, Error> {
    Ok(reqwest::get(url).await?.text().await?)
}
```

```rust
// GOOD
pub async fn fetch(url: &str) -> Result<String, Error> {
    let response = timeout(
        Duration::from_secs(10),
        reqwest::get(url)
    ).await??;
    
    Ok(response.text().await?)
}
```

### ❌ Unbounded task spawning

```rust
// BAD
pub async fn process_all(items: Vec<Item>) -> Vec<Output> {
    let mut tasks = Vec::new();
    for item in items {
        tasks.push(tokio::spawn(process(item))); // Could spawn millions!
    }
    // ...
}
```

```rust
// GOOD
pub async fn process_all(items: Vec<Item>) -> Vec<Output> {
    stream::iter(items)
        .map(|item| process(item))
        .buffer_unordered(10) // Bounded concurrency
        .collect()
        .await
}
```

### ❌ Ignoring cancellation

```rust
// BAD
pub async fn long_running_task() {
    loop {
        do_work().await;
        // No way to cancel!
    }
}
```

```rust
// GOOD
pub async fn long_running_task(mut shutdown: watch::Receiver<bool>) {
    loop {
        select! {
            _ = shutdown.changed() => break,
            _ = do_work() => {}
        }
    }
}
```

## Testing Async Code

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{pause, advance, Duration};

    #[tokio::test]
    async fn test_timeout_works() {
        let result = timeout(
            Duration::from_millis(100),
            async {
                tokio::time::sleep(Duration::from_secs(10)).await;
                "done"
            }
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_processing() {
        let items = vec![1, 2, 3, 4, 5];
        let results = process_batch(items).await.unwrap();
        assert_eq!(results.len(), 5);
    }
}
```

## Observability

Instrument async functions for tracing:

```rust
use tracing::{instrument, info, warn};

#[instrument(skip(data))]
pub async fn process_chart(
    chart_id: &str,
    data: Vec<u8>,
) -> Result<RoxChart, Error> {
    info!("Starting chart processing");
    
    let chart = tokio::task::spawn_blocking(move || {
        decode_chart(&data)
    })
    .await
    .map_err(|e| {
        warn!(error = %e, "Blocking task failed");
        Error::DecodeFailed
    })??;
    
    info!(note_count = chart.notes.len(), "Chart processed");
    Ok(chart)
}
```

## Checklist

When implementing async code:

- [ ] CPU-bound work uses `spawn_blocking`
- [ ] All I/O operations have timeouts
- [ ] Concurrent operations are bounded
- [ ] Proper error handling with `?` and `Result`
- [ ] Cancellation is supported where needed
- [ ] Resource limits are explicit and documented
- [ ] Async functions are instrumented for observability
- [ ] Tests cover timeout and cancellation scenarios

## References

- User rule: `rule-memory-and-ressources.md` (Resource & Memory Budget)
- User rule: `rule-observability.md` (Observability)
- [Tokio documentation](https://docs.rs/tokio/)
- [Async Rust Book](https://rust-lang.github.io/async-book/)
