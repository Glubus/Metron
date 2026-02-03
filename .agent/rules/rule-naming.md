---
trigger: always_on
---

# Naming Conventions Rule

## Concise Lesson

Strictly adhere to Rust standards for readability and maintenance:

1. **Variables & Functions**: `snake_case` (e.g., `calculate_width`, `user_id`).
2. **Types & Traits**: `PascalCase` (e.g., `TextureManager`, `ResizeStrategy`).
3. **Constants & Statics**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_BUFFER_SIZE`).
4. **Files**: `snake_case.rs` (e.g., `image_processor.rs`).
5. **Explicit**: Prefer long and descriptive names (`user_account_id`) over cryptic abbreviations (`uid`).

## Good Example

```rust
const MAX_RETRIES: u32 = 3;

struct UserProfile {
    user_name: String,
    login_count: u32,
}

impl UserProfile {
    fn reset_stats(&mut self) {
        self.login_count = 0;
    }
}
```

## Bad Example

```rust
const maxRetries: u32 = 3; // BAD: camelCase

struct user_profile { // BAD: snake_case for struct
    UserName: String, // BAD: PascalCase for field
    l_cnt: u32, // BAD: Cryptic abbreviation
}

fn ResetStats() {} // BAD: PascalCase for function
```