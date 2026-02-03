---
trigger: always_on
---

# Rust Strict Coding Standards & Workflow

## Concise Lesson

1. **Structural Limits**:
    * Max **40 lines** per function.
    * Max **150 lines** per file.
    * *Separate responsibilities*: 1 File = 1 Concept.
2. **Robustness & Errors**:
    * **Never use `unwrap()`** or `expect()` except in test.
    * Always propagate errors cleanly with `Result<T, E>`.
    * Use domain-specific error types (e.g., `thiserror`), no `anyhow` for core logic.
    * Reflect when to use match or if let() (match should used for more than 2) 
3. **Workflow & Documentation**:
    * **TDD (Test Driven Development)**: Write the test *before* the implementation.
    * **Pipeline Thinking**: Visualize the data flow.
    * **Immediate Documentation**: Document the *Why* of choices (libs, algos) in comments above the function AND in `wiki.wiki`.

## Good Example

```rust
use thiserror::Error;

/// Domain-specific errors for resizing
#[derive(Error, Debug)]
pub enum ResizeError {
    #[error("Invalid dimension: {0}")]
    InvalidDimension(u32),
    #[error("Decode error: {0}")]
    DecodeError(#[from] image::ImageError),
}

/// Calculates new dimensions (TDD First)
/// Lib Choice: No external lib used for simple calculation to avoid unnecessary deps.
/// Ref: wiki.wiki#image-scaling-algo
fn calculate_dimensions(w: u32, h: u32, target_w: u32) -> Result<(u32, u32), ResizeError> {
    if w == 0 || h == 0 {
        return Err(ResizeError::InvalidDimension(0));
    }
    let ratio = target_w as f32 / w as f32;
    Ok((target_w, (h as f32 * ratio) as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_dimensions_nominal() {
        let (w, h) = calculate_dimensions(100, 50, 50).unwrap();
        assert_eq!(w, 50);
        assert_eq!(h, 25);
    }
    
    #[test]
    fn test_calculate_dimensions_zero_error() {
        assert!(calculate_dimensions(0, 50, 50).is_err());
    }
}
```

## Bad Example

```rust
// File with 300 lines, huge 80-line function
fn process_image(path: &str) -> anyhow::Result<()> {
    let img = image::open(path).unwrap(); // Panic risk!
    // ... complex mixed logic ...
    let res = do_something_complex(img); // No test, no doc
    Ok(())
}
```
