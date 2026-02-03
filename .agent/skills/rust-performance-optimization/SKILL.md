---
name: rust-performance-optimization
description: Performance optimization patterns for Rust. Use when optimizing hot paths, reducing allocations, improving cache locality, or benchmarking code. Focuses on data-oriented design and zero-cost abstractions.
---

# Rust Performance Optimization Skill

This skill provides guidance on writing high-performance Rust code through profiling, optimization, and data-oriented design.

## When to use this skill

- Optimizing hot paths identified by profiling
- Reducing memory allocations
- Improving cache locality
- Writing benchmarks
- Analyzing performance bottlenecks
- Optimizing binary size or compile times

## Core Principles

### 1. Profile Before Optimizing

**Rule**: Never optimize without profiling data. Measure first, optimize second.

**Tools**:

- `cargo bench` for microbenchmarks
- `cargo flamegraph` for CPU profiling
- `heaptrack` or `valgrind` for memory profiling
- `perf` for Linux system profiling

```rust
// benches/decode_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_decode(c: &mut Criterion) {
    let data = std::fs::read("assets/test.osu").unwrap();
    
    c.bench_function("decode_osu", |b| {
        b.iter(|| {
            decode_osu(black_box(&data))
        });
    });
}

criterion_group!(benches, benchmark_decode);
criterion_main!(benches);
```

### 2. Minimize Allocations

**Rule**: Allocations are expensive. Reuse buffers, use stack allocation when possible, and avoid unnecessary clones.

**Bad**:

```rust
pub fn process_notes(notes: &[Note]) -> Vec<Note> {
    let mut result = Vec::new();
    for note in notes {
        if note.time > 0 {
            result.push(note.clone()); // Allocation + clone
        }
    }
    result
}
```

**Good**:

```rust
pub fn process_notes(notes: &[Note], output: &mut Vec<Note>) {
    output.clear();
    output.reserve(notes.len()); // Pre-allocate
    
    for note in notes {
        if note.time > 0 {
            output.push(*note); // Copy (if Note is Copy)
        }
    }
}
```

**Better** (zero allocation):

```rust
pub fn process_notes_iter(notes: &[Note]) -> impl Iterator<Item = &Note> {
    notes.iter().filter(|n| n.time > 0)
}
```

### 3. Use Copy Types for Small Data

**Rule**: Types ≤ 16 bytes should implement `Copy` for performance.

```rust
/// Small, Copy-able note type
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Note {
    pub time: i64,      // 8 bytes
    pub column: u8,     // 1 byte
    pub note_type: u8,  // 1 byte
    pub duration: u32,  // 4 bytes
    // Total: 14 bytes (+ 2 padding = 16)
}
```

### 4. Optimize Data Layout

**Rule**: Arrange struct fields for cache efficiency and minimal padding.

**Bad**:

```rust
struct Chart {
    key_count: u8,        // 1 byte
    metadata: Metadata,   // Large
    hash: [u8; 32],       // 32 bytes
    notes: Vec<Note>,     // 24 bytes
    is_valid: bool,       // 1 byte
}
```

**Good**:

```rust
struct Chart {
    // Hot data first (frequently accessed together)
    notes: Vec<Note>,     // 24 bytes
    key_count: u8,        // 1 byte
    is_valid: bool,       // 1 byte
    _padding: [u8; 6],    // Explicit padding
    
    // Cold data last
    hash: [u8; 32],
    metadata: Metadata,
}
```

## Optimization Patterns

### Pattern 1: Buffer Reuse

```rust
pub struct ChartProcessor {
    // Reusable buffers
    note_buffer: Vec<Note>,
    temp_buffer: Vec<u8>,
}

impl ChartProcessor {
    pub fn process(&mut self, chart: &RoxChart) -> Result<ProcessedChart, Error> {
        // Reuse existing allocation
        self.note_buffer.clear();
        self.note_buffer.extend_from_slice(&chart.notes);
        
        // Process in-place
        self.note_buffer.sort_unstable_by_key(|n| n.time);
        
        Ok(ProcessedChart {
            notes: self.note_buffer.clone(),
        })
    }
}
```

### Pattern 2: Small String Optimization

```rust
use smartstring::alias::String as SmartString;

/// Uses stack storage for strings ≤ 23 bytes
pub struct Metadata {
    pub title: SmartString,
    pub artist: SmartString,
    pub creator: SmartString,
}
```

### Pattern 3: Lazy Computation

```rust
pub struct RoxChart {
    notes: Vec<Note>,
    
    // Cached, computed on demand
    #[serde(skip)]
    cached_hash: OnceCell<Blake3Hash>,
}

impl RoxChart {
    pub fn hash(&self) -> &Blake3Hash {
        self.cached_hash.get_or_init(|| {
            compute_hash(&self.notes)
        })
    }
}
```

### Pattern 4: SIMD for Bulk Operations

```rust
use std::simd::{f32x8, SimdFloat};

/// Vectorized note time scaling
pub fn scale_note_times_simd(notes: &mut [Note], factor: f32) {
    let factor_vec = f32x8::splat(factor);
    
    // Process 8 notes at a time
    for chunk in notes.chunks_exact_mut(8) {
        let times: [f32; 8] = chunk.iter()
            .map(|n| n.time as f32)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        
        let times_vec = f32x8::from_array(times);
        let scaled = times_vec * factor_vec;
        
        for (note, &time) in chunk.iter_mut().zip(scaled.as_array()) {
            note.time = time as i64;
        }
    }
    
    // Handle remainder
    for note in notes.chunks_exact_mut(8).into_remainder() {
        note.time = (note.time as f32 * factor) as i64;
    }
}
```

### Pattern 5: Const Evaluation

```rust
/// Computed at compile time
const MAX_NOTES: usize = 100_000;
const NOTE_SIZE: usize = std::mem::size_of::<Note>();
const MAX_CHART_SIZE: usize = MAX_NOTES * NOTE_SIZE;

/// Compile-time lookup table
const BPM_TO_MS: [f64; 300] = {
    let mut table = [0.0; 300];
    let mut i = 0;
    while i < 300 {
        table[i] = 60_000.0 / (i as f64 + 1.0);
        i += 1;
    }
    table
};
```

## Profiling Workflow

### 1. Create Benchmark

```bash
# Add to Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "chart_processing"
harness = false
```

### 2. Run Benchmark

```bash
cargo bench --bench chart_processing
```

### 3. Profile with Flamegraph

```bash
cargo install flamegraph
cargo flamegraph --bench chart_processing
```

### 4. Analyze Results

Look for:

- Hot functions (wide bars in flamegraph)
- Unexpected allocations
- Cache misses
- Lock contention

### 5. Optimize and Re-measure

```bash
# Before optimization
cargo bench -- --save-baseline before

# After optimization
cargo bench -- --baseline before
```

## Common Optimizations

### Reduce String Allocations

**Before**:

```rust
pub fn format_time(time_us: i64) -> String {
    format!("{}:{:02}", time_us / 60_000_000, (time_us / 1_000_000) % 60)
}
```

**After**:

```rust
use std::fmt::Write;

pub fn format_time(time_us: i64, buffer: &mut String) {
    buffer.clear();
    write!(
        buffer,
        "{}:{:02}",
        time_us / 60_000_000,
        (time_us / 1_000_000) % 60
    ).unwrap();
}
```

### Use `&str` Instead of `String` Where Possible

**Before**:

```rust
pub fn parse_metadata(data: String) -> Metadata {
    // ...
}
```

**After**:

```rust
pub fn parse_metadata(data: &str) -> Metadata {
    // No allocation needed
}
```

### Avoid Unnecessary Clones

**Before**:

```rust
pub fn get_notes(&self) -> Vec<Note> {
    self.notes.clone() // Expensive!
}
```

**After**:

```rust
pub fn notes(&self) -> &[Note] {
    &self.notes // Zero-cost
}
```

### Use `Cow` for Conditional Ownership

```rust
use std::borrow::Cow;

pub fn normalize_title(title: &str) -> Cow<str> {
    if title.chars().all(|c| c.is_ascii()) {
        Cow::Borrowed(title) // No allocation
    } else {
        Cow::Owned(title.to_lowercase()) // Allocate only if needed
    }
}
```

## Memory Layout Optimization

### Check Struct Size

```rust
#[test]
fn test_struct_sizes() {
    use std::mem::size_of;
    
    assert_eq!(size_of::<Note>(), 16);
    assert_eq!(size_of::<TimingPoint>(), 24);
    
    // Ensure no unexpected padding
    println!("Note size: {}", size_of::<Note>());
}
```

### Use `#[repr(C)]` for Predictable Layout

```rust
#[repr(C)]
pub struct Note {
    pub time: i64,
    pub column: u8,
    pub note_type: u8,
    pub duration: u32,
}
```

### Consider Enum Discriminant Size

```rust
// Bad: Large discriminant
enum BadNote {
    Tap { time: i64, column: u8 },
    Hold { time: i64, column: u8, duration: u32 },
}
// Size: 24 bytes (8 discriminant + 16 data)

// Good: Compact representation
#[repr(u8)]
enum NoteType {
    Tap = 0,
    Hold = 1,
}

struct GoodNote {
    time: i64,
    column: u8,
    note_type: NoteType,
    duration: u32, // 0 for taps
}
// Size: 16 bytes
```

## Compile-Time Optimization

### Enable LTO (Link-Time Optimization)

```toml
# Cargo.toml
[profile.release]
lto = "fat"
codegen-units = 1
opt-level = 3
```

### Reduce Binary Size

```toml
[profile.release]
strip = true
lto = true
opt-level = "z"  # Optimize for size
codegen-units = 1
panic = "abort"
```

## Common Mistakes to Avoid

### ❌ Premature Optimization

```rust
// BAD: Optimizing without profiling
pub fn process(data: &[u8]) -> Vec<u8> {
    // Complex, unreadable optimization
    // that doesn't help performance
}
```

### ❌ Over-allocating

```rust
// BAD
let mut vec = Vec::with_capacity(1_000_000);
vec.push(1); // Only using 1 element!
```

```rust
// GOOD
let mut vec = Vec::with_capacity(expected_size);
```

### ❌ Unnecessary Sorting

```rust
// BAD: Sorting when not needed
notes.sort();
let first = notes.first();
```

```rust
// GOOD: Use min/max
let first = notes.iter().min_by_key(|n| n.time);
```

### ❌ String Concatenation in Loops

```rust
// BAD
let mut result = String::new();
for item in items {
    result = result + &item.to_string(); // Reallocates each time!
}
```

```rust
// GOOD
let mut result = String::with_capacity(estimated_size);
for item in items {
    write!(&mut result, "{}", item).unwrap();
}
```

## Benchmarking Best Practices

```rust
use criterion::{black_box, Criterion, BenchmarkId};

fn benchmark_decode_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");
    
    for size in [100, 1_000, 10_000, 50_000] {
        let data = generate_test_chart(size);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &data,
            |b, data| {
                b.iter(|| decode_chart(black_box(data)))
            },
        );
    }
    
    group.finish();
}
```

## Checklist

When optimizing performance:

- [ ] Profile first to identify bottlenecks
- [ ] Write benchmarks before optimizing
- [ ] Minimize allocations in hot paths
- [ ] Use `Copy` types for small data
- [ ] Optimize struct layout for cache locality
- [ ] Reuse buffers where possible
- [ ] Use iterators instead of intermediate collections
- [ ] Enable LTO for release builds
- [ ] Measure improvement with benchmarks
- [ ] Document why optimization was needed

## References

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Criterion.rs](https://github.com/bheisler/criterion.rs)
- User rule: `rule-memory-and-ressources.md`
- [Flamegraph](https://github.com/flamegraph-rs/flamegraph)
