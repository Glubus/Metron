---
name: rust-error-handling
description: Comprehensive error handling patterns for Rust. Use when creating error types, handling Results, or implementing error propagation. Enforces domain-specific errors with thiserror, zero unwrap() usage, and proper error context.
---

# Rust Error Handling Skill

This skill guides proper error handling in Rust projects, enforcing strict standards for robustness and maintainability.

## When to use this skill

- Creating new error types for a module or domain
- Handling `Result<T, E>` types
- Converting between error types
- Implementing error propagation
- Reviewing code for error handling issues

## Core Principles

### 1. Never Use `unwrap()` or `expect()` in Production Code

**Rule**: `unwrap()` and `expect()` are **forbidden** in production code. They are only allowed in:

- Test code (`#[cfg(test)]`)
- Example code
- Prototypes explicitly marked as such

**Why**: These methods cause panics, which are unrecoverable in production. All errors must be handled gracefully.

### 2. Use Domain-Specific Error Types

**Rule**: Every module or domain should have its own error type using `thiserror`.

**Why**:

- Clear error semantics
- Type-safe error handling
- Better error messages
- Easier debugging

**Example**:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodecError {
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("Invalid key count: {0} (expected 1-18)")]
    InvalidKeyCount(u8),
    
    #[error("Decode failed: {0}")]
    DecodeFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### 3. Avoid `anyhow` in Library Code

**Rule**:

- Use `thiserror` for library/core logic
- `anyhow` is acceptable only for CLI tools and applications

**Why**: Libraries should expose typed errors for downstream consumers.

### 4. Provide Context with Error Messages

**Rule**: Error messages must include:

- What went wrong
- Relevant values (sanitized if sensitive)
- Context about the operation

**Good**:

```rust
return Err(CodecError::InvalidKeyCount(key_count));
```

**Bad**:

```rust
return Err(CodecError::Invalid); // No context!
```

## Error Handling Patterns

### Pattern 1: Early Return with `?`

```rust
pub fn decode_chart(path: &Path) -> Result<RoxChart, CodecError> {
    let data = std::fs::read(path)?; // Auto-converts io::Error
    let chart = parse_data(&data)?;
    validate_chart(&chart)?;
    Ok(chart)
}
```

### Pattern 2: Match for Multiple Error Paths

```rust
pub fn load_audio(path: &Path) -> Result<AudioData, AudioError> {
    match std::fs::read(path) {
        Ok(data) => {
            if data.is_empty() {
                Err(AudioError::EmptyFile)
            } else {
                Ok(AudioData::from_bytes(data))
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(AudioError::FileNotFound(path.to_path_buf()))
        }
        Err(e) => Err(AudioError::Io(e)),
    }
}
```

### Pattern 3: Custom Error Conversion

```rust
impl From<rkyv::validation::CheckArchiveError<_, _>> for CodecError {
    fn from(err: rkyv::validation::CheckArchiveError<_, _>) -> Self {
        CodecError::DecodeFailed(format!("Archive validation failed: {}", err))
    }
}
```

### Pattern 4: Result Chaining with `and_then`

```rust
pub fn process_chart(path: &Path) -> Result<ProcessedChart, CodecError> {
    load_chart(path)
        .and_then(|chart| validate_timing(&chart).map(|_| chart))
        .and_then(|chart| normalize_notes(chart))
}
```

## Decision Tree

### When creating a new error type

1. **Is this a library module?**
   - Yes → Use `thiserror`, create domain-specific enum
   - No (CLI/app) → Consider `anyhow` for simplicity

2. **Do I need to convert from other error types?**
   - Yes → Use `#[from]` attribute in `thiserror`
   - No → Manual `From` impl if needed

3. **Do I need additional context?**
   - Yes → Add fields to error variants
   - No → Simple message-only variants

### When handling errors

1. **Can I recover from this error?**
   - Yes → Use `match` or `if let` to handle specific cases
   - No → Propagate with `?`

2. **Do I need to add context?**
   - Yes → Use `.map_err()` to wrap or annotate
   - No → Direct propagation with `?`

3. **Is this a validation error?**
   - Yes → Return early with descriptive error
   - No → Continue processing

## Common Mistakes to Avoid

### ❌ Using `unwrap()` in production

```rust
// BAD
let chart = decode_chart(path).unwrap();
```

```rust
// GOOD
let chart = decode_chart(path)?;
```

### ❌ Generic error messages

```rust
// BAD
#[error("Error occurred")]
InvalidInput,
```

```rust
// GOOD
#[error("Invalid key count: {0} (expected 1-18)")]
InvalidKeyCount(u8),
```

### ❌ Swallowing errors silently

```rust
// BAD
if let Err(_) = process() {
    // Silent failure
}
```

```rust
// GOOD
if let Err(e) = process() {
    error!("Processing failed: {}", e);
    return Err(e.into());
}
```

### ❌ Using `String` for errors

```rust
// BAD
pub fn decode(data: &[u8]) -> Result<Chart, String> {
    // ...
}
```

```rust
// GOOD
pub fn decode(data: &[u8]) -> Result<Chart, CodecError> {
    // ...
}
```

## Testing Error Paths

Always test error conditions:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_key_count() {
        let result = validate_key_count(0);
        assert!(matches!(result, Err(CodecError::InvalidKeyCount(0))));
    }

    #[test]
    fn test_file_not_found() {
        let result = load_chart(Path::new("nonexistent.rox"));
        assert!(result.is_err());
    }
}
```

## Integration with Observability

Errors should be logged with context:

```rust
use tracing::{error, warn};

pub fn process_batch(charts: &[Path]) -> Result<Vec<RoxChart>, CodecError> {
    let mut results = Vec::new();
    
    for path in charts {
        match decode_chart(path) {
            Ok(chart) => results.push(chart),
            Err(e) => {
                error!(path = ?path, error = %e, "Failed to decode chart");
                return Err(e);
            }
        }
    }
    
    Ok(results)
}
```

## Checklist

When implementing error handling, ensure:

- [ ] No `unwrap()` or `expect()` in production code
- [ ] Domain-specific error type using `thiserror`
- [ ] All error variants have descriptive messages
- [ ] Error conversions use `#[from]` where appropriate
- [ ] Errors include relevant context (values, paths, etc.)
- [ ] Error paths are tested
- [ ] Errors are logged with appropriate levels
- [ ] Public API errors are documented

## References

- User rule: `rule-input.md` (Input Validation & Trust Boundary)
- User rule: `rust-strict-standards.md` (Robustness & Errors section)
- [thiserror documentation](https://docs.rs/thiserror/)
- [Rust Error Handling Book](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
