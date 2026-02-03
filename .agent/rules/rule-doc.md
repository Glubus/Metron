---
trigger: always_on
---

# Documentation Standards Rule

## Concise Lesson

Documentation is not optional; it is part of the code:

1. **Immediate**: Document a function *while* you write it, not after.
2. **Focus on "Why"**: Explain *why* this approach was chosen, *why* this library, *why* this algo. The "how" is already in the code.
3. **Doc Comments**: Use `///` to document public functions and structs.
4. **Wiki Link**: Refer to architectural decisions listed in `wiki.wiki` if necessary.

## Good Example (in english but do it in french)

```rust
/// Resizes the image using the Lanczos3 algorithm.
/// 
/// # Why Lanczos3?
/// We chose Lanczos3 because it offers the best quality/precision compromise for
/// the "photo" type images we process, despite a slightly higher CPU cost
/// than `Triangle`. See `wiki.wiki` section "Image Scaling".
pub fn resize_high_quality(img: &DynamicImage, w: u32, h: u32) -> DynamicImage {
    // ...
}
```

## Bad Example

```rust
// Resize function
// args: img, w, h
fn resize(img: &DynamicImage, w: u32, h: u32) {
    // code
}
```

*Critique: Redundant comment, does not explain choices, incorrect format for automatic docs.*