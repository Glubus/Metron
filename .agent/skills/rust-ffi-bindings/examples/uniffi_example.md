# UniFFI Example - Complete Setup

This example shows how to set up UniFFI for multi-language bindings.

## Project Structure

```
bindings/uniffi/
├── Cargo.toml
├── build.rs
├── src/
│   ├── lib.rs
│   └── rox.udl (optional, if using UDL instead of macros)
└── examples/
    ├── kotlin_example.kt
    ├── swift_example.swift
    └── python_example.py
```

## Cargo.toml

```toml
[package]
name = "rox-uniffi"
version = "0.1.0"
edition = "2021"

[dependencies]
uniffi = "0.25"
thiserror = "1.0"

# Your core library
rhythm-open-exchange = { path = "../../" }

[build-dependencies]
uniffi = { version = "0.25", features = ["build"] }

[lib]
crate-type = ["cdylib", "staticlib"]
name = "rox_uniffi"
```

## build.rs

```rust
fn main() {
    // If using UDL file
    uniffi::generate_scaffolding("src/rox.udl").unwrap();
    
    // If using proc macros only, you can skip this
}
```

## src/lib.rs (Macro Approach)

```rust
use uniffi;

#[derive(uniffi::Object)]
pub struct RoxChart {
    inner: rhythm_open_exchange::RoxChart,
}

#[uniffi::export]
impl RoxChart {
    #[uniffi::constructor]
    pub fn new(key_count: u8) -> Result<Self, RoxError> {
        if key_count == 0 || key_count > 18 {
            return Err(RoxError::InvalidKeyCount { count: key_count });
        }
        
        Ok(Self {
            inner: rhythm_open_exchange::RoxChart::new(key_count),
        })
    }
    
    pub fn add_tap_note(&mut self, time_us: i64, column: u8) -> Result<(), RoxError> {
        if column >= self.inner.key_count {
            return Err(RoxError::InvalidColumn { 
                column, 
                max: self.inner.key_count 
            });
        }
        
        self.inner.notes.push(rhythm_open_exchange::Note::tap(time_us, column));
        Ok(())
    }
    
    pub fn add_hold_note(
        &mut self, 
        time_us: i64, 
        duration_us: u32, 
        column: u8
    ) -> Result<(), RoxError> {
        if column >= self.inner.key_count {
            return Err(RoxError::InvalidColumn { 
                column, 
                max: self.inner.key_count 
            });
        }
        
        self.inner.notes.push(
            rhythm_open_exchange::Note::hold(time_us, duration_us, column)
        );
        Ok(())
    }
    
    pub fn get_metadata(&self) -> Metadata {
        Metadata {
            title: self.inner.metadata.title.clone(),
            artist: self.inner.metadata.artist.clone(),
            creator: self.inner.metadata.creator.clone(),
            difficulty_name: self.inner.metadata.difficulty_name.clone(),
            audio_file: self.inner.metadata.audio_file.clone(),
        }
    }
    
    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.inner.metadata.title = metadata.title;
        self.inner.metadata.artist = metadata.artist;
        self.inner.metadata.creator = metadata.creator;
        self.inner.metadata.difficulty_name = metadata.difficulty_name;
        self.inner.metadata.audio_file = metadata.audio_file;
    }
    
    pub fn note_count(&self) -> u32 {
        self.inner.notes.len() as u32
    }
    
    pub fn key_count(&self) -> u8 {
        self.inner.key_count
    }
}

#[derive(uniffi::Record)]
pub struct Metadata {
    pub title: String,
    pub artist: String,
    pub creator: String,
    pub difficulty_name: String,
    pub audio_file: String,
}

#[derive(uniffi::Error, Debug, thiserror::Error)]
pub enum RoxError {
    #[error("Invalid key count: {count} (expected 1-18)")]
    InvalidKeyCount { count: u8 },
    
    #[error("Invalid column {column} (max {max})")]
    InvalidColumn { column: u8, max: u8 },
    
    #[error("Decode failed: {reason}")]
    DecodeFailed { reason: String },
    
    #[error("Encode failed: {reason}")]
    EncodeFailed { reason: String },
}

// Top-level functions
#[uniffi::export]
pub fn create_chart(key_count: u8) -> Result<RoxChart, RoxError> {
    RoxChart::new(key_count)
}

#[uniffi::export]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Generate scaffolding
uniffi::setup_scaffolding!();
```

## Building for Different Platforms

### Kotlin (Android)

```bash
# Build the library
cargo build --release --target aarch64-linux-android

# Generate Kotlin bindings
cargo run --bin uniffi-bindgen generate \
    src/rox.udl \
    --language kotlin \
    --out-dir ./bindings/kotlin
```

### Swift (iOS)

```bash
# Build for iOS
cargo build --release --target aarch64-apple-ios

# Generate Swift bindings
cargo run --bin uniffi-bindgen generate \
    src/rox.udl \
    --language swift \
    --out-dir ./bindings/swift
```

### Python

```bash
# Build the library
cargo build --release

# Generate Python bindings
cargo run --bin uniffi-bindgen generate \
    src/rox.udl \
    --language python \
    --out-dir ./bindings/python

# Create wheel
cd bindings/python
python setup.py bdist_wheel
```

## Usage Examples

### Kotlin

```kotlin
import uniffi.rox_uniffi.*

fun main() {
    val chart = createChart(4u)
    
    chart.addTapNote(1000000, 0u)
    chart.addHoldNote(2000000, 500000u, 1u)
    
    val metadata = Metadata(
        title = "My Song",
        artist = "Artist Name",
        creator = "Mapper",
        difficultyName = "Hard",
        audioFile = "audio.ogg"
    )
    chart.setMetadata(metadata)
    
    println("Chart has ${chart.noteCount()} notes")
}
```

### Swift

```swift
import rox_uniffi

let chart = try createChart(keyCount: 4)

try chart.addTapNote(timeUs: 1000000, column: 0)
try chart.addHoldNote(timeUs: 2000000, durationUs: 500000, column: 1)

let metadata = Metadata(
    title: "My Song",
    artist: "Artist Name",
    creator: "Mapper",
    difficultyName: "Hard",
    audioFile: "audio.ogg"
)
chart.setMetadata(metadata: metadata)

print("Chart has \(chart.noteCount()) notes")
```

### Python

```python
from rox_uniffi import create_chart, Metadata

chart = create_chart(4)

chart.add_tap_note(1000000, 0)
chart.add_hold_note(2000000, 500000, 1)

metadata = Metadata(
    title="My Song",
    artist="Artist Name",
    creator="Mapper",
    difficulty_name="Hard",
    audio_file="audio.ogg"
)
chart.set_metadata(metadata)

print(f"Chart has {chart.note_count()} notes")
```

## Tips

1. **Use macros for simple APIs**: The macro approach is cleaner for straightforward APIs
2. **Use UDL for complex APIs**: UDL gives more control for complex type hierarchies
3. **Keep types simple**: Avoid complex generics or lifetimes in UniFFI types
4. **Test on all platforms**: UniFFI generates different code per platform
5. **Version carefully**: Breaking changes in Rust affect all language bindings
