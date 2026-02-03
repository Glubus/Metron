---
name: rust-ffi-bindings
description: FFI and language bindings patterns for Rust. Use when creating C APIs, C# bindings, Python bindings, WebAssembly exports, or multi-language bindings with UniFFI. Enforces safety, ABI stability, and proper memory management across language boundaries.
---

# Rust FFI & Bindings Skill

This skill provides guidance on creating safe, stable, and ergonomic Foreign Function Interface (FFI) bindings for Rust libraries.

## When to use this skill

- Creating C API exports from Rust
- Building C# / .NET bindings
- Implementing Python bindings (PyO3)
- Creating WebAssembly bindings
- Using UniFFI for multi-language bindings (Kotlin, Swift, Python, Ruby)
- Exposing Rust functionality to other languages
- Managing memory across language boundaries

## Core Principles

### 1. Safety at the Boundary

**Rule**: All FFI functions must validate inputs and handle panics.

**Why**: Panics across FFI boundaries cause undefined behavior.

```rust
#[no_mangle]
pub extern "C" fn rox_decode_chart(
    data_ptr: *const u8,
    data_len: usize,
    out_chart: *mut *mut RoxChart,
) -> i32 {
    // Catch panics
    let result = std::panic::catch_unwind(|| {
        // Validate inputs
        if data_ptr.is_null() || out_chart.is_null() {
            return -1; // Error code
        }
        
        // Safe conversion
        let data = unsafe {
            std::slice::from_raw_parts(data_ptr, data_len)
        };
        
        // Actual work
        match decode_chart(data) {
            Ok(chart) => {
                unsafe {
                    *out_chart = Box::into_raw(Box::new(chart));
                }
                0 // Success
            }
            Err(_) => -2, // Decode error
        }
    });
    
    result.unwrap_or(-3) // Panic occurred
}
```

### 2. ABI Stability

**Rule**: Use `#[repr(C)]` for all exported structs and enums.

**Why**: Ensures consistent memory layout across compilers and languages.

```rust
/// Stable C-compatible representation
#[repr(C)]
pub struct RoxChartHandle {
    /// Opaque pointer to internal Rust type
    inner: *mut RoxChart,
}

#[repr(C)]
pub enum RoxFormat {
    Rox = 0,
    Osu = 1,
    StepMania = 2,
    Quaver = 3,
}
```

### 3. Explicit Memory Ownership

**Rule**: Document and enforce ownership semantics for every FFI function.

**Patterns**:

- **Borrow**: Caller owns, function borrows temporarily
- **Transfer**: Ownership transfers to/from Rust
- **Clone**: Function creates independent copy

```rust
/// Borrows data (caller retains ownership)
#[no_mangle]
pub extern "C" fn rox_chart_get_title(
    chart: *const RoxChart,
    out_ptr: *mut *const c_char,
) -> i32 {
    // Function borrows, caller still owns chart
}

/// Transfers ownership TO Rust (caller must not free)
#[no_mangle]
pub extern "C" fn rox_chart_create(
    key_count: u8,
) -> *mut RoxChart {
    Box::into_raw(Box::new(RoxChart::new(key_count)))
}

/// Transfers ownership FROM Rust (caller must free)
#[no_mangle]
pub extern "C" fn rox_chart_destroy(chart: *mut RoxChart) {
    if !chart.is_null() {
        unsafe {
            let _ = Box::from_raw(chart);
        }
    }
}
```

### 4. Error Handling Across FFI

**Rule**: FFI functions return error codes or status, never panic.

```rust
/// Error codes for FFI
#[repr(C)]
pub enum RoxError {
    Success = 0,
    NullPointer = -1,
    InvalidInput = -2,
    DecodeFailed = -3,
    OutOfMemory = -4,
    Panic = -99,
}

#[no_mangle]
pub extern "C" fn rox_get_last_error(
    buffer: *mut c_char,
    buffer_len: usize,
) -> i32 {
    // Thread-local error storage
    LAST_ERROR.with(|e| {
        if let Some(ref err) = *e.borrow() {
            let err_str = err.to_string();
            let bytes = err_str.as_bytes();
            let copy_len = bytes.len().min(buffer_len - 1);
            
            unsafe {
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    buffer as *mut u8,
                    copy_len,
                );
                *buffer.add(copy_len) = 0; // Null terminator
            }
            
            copy_len as i32
        } else {
            0
        }
    })
}
```

## Language-Specific Patterns

### C API Pattern

```rust
// lib.rs or ffi.rs

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Opaque handle for C
pub struct RoxChartHandle {
    _private: [u8; 0],
}

#[no_mangle]
pub extern "C" fn rox_chart_new(key_count: u8) -> *mut RoxChartHandle {
    let chart = Box::new(RoxChart::new(key_count));
    Box::into_raw(chart) as *mut RoxChartHandle
}

#[no_mangle]
pub extern "C" fn rox_chart_free(handle: *mut RoxChartHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle as *mut RoxChart);
        }
    }
}

#[no_mangle]
pub extern "C" fn rox_chart_add_note(
    handle: *mut RoxChartHandle,
    time_us: i64,
    column: u8,
) -> i32 {
    if handle.is_null() {
        return RoxError::NullPointer as i32;
    }
    
    let chart = unsafe { &mut *(handle as *mut RoxChart) };
    chart.notes.push(Note::tap(time_us, column));
    
    RoxError::Success as i32
}
```

### C# Bindings Pattern (using CsBindgen or manual P/Invoke)

```rust
// For C# interop, use simple types and explicit layouts

#[repr(C)]
pub struct RoxMetadata {
    pub title: *const c_char,
    pub artist: *const c_char,
    pub creator: *const c_char,
    pub difficulty_name: *const c_char,
    pub audio_file: *const c_char,
}

#[no_mangle]
pub extern "C" fn rox_chart_get_metadata(
    chart: *const RoxChart,
    out_metadata: *mut RoxMetadata,
) -> i32 {
    if chart.is_null() || out_metadata.is_null() {
        return -1;
    }
    
    let chart = unsafe { &*chart };
    
    // Allocate C strings (caller must free)
    let metadata = RoxMetadata {
        title: CString::new(chart.metadata.title.as_str())
            .unwrap()
            .into_raw(),
        artist: CString::new(chart.metadata.artist.as_str())
            .unwrap()
            .into_raw(),
        // ... other fields
    };
    
    unsafe {
        *out_metadata = metadata;
    }
    
    0
}

#[no_mangle]
pub extern "C" fn rox_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}
```

### Python Bindings Pattern (PyO3)

```rust
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

#[pyclass]
pub struct PyRoxChart {
    inner: RoxChart,
}

#[pymethods]
impl PyRoxChart {
    #[new]
    fn new(key_count: u8) -> PyResult<Self> {
        if key_count == 0 || key_count > 18 {
            return Err(PyValueError::new_err(
                "Key count must be between 1 and 18"
            ));
        }
        
        Ok(Self {
            inner: RoxChart::new(key_count),
        })
    }
    
    fn add_note(&mut self, time_us: i64, column: u8) -> PyResult<()> {
        if column >= self.inner.key_count {
            return Err(PyValueError::new_err(
                format!("Column {} out of range", column)
            ));
        }
        
        self.inner.notes.push(Note::tap(time_us, column));
        Ok(())
    }
    
    #[getter]
    fn title(&self) -> String {
        self.inner.metadata.title.clone()
    }
    
    #[setter]
    fn set_title(&mut self, title: String) {
        self.inner.metadata.title = title;
    }
    
    fn __repr__(&self) -> String {
        format!(
            "RoxChart({}K, {} notes)",
            self.inner.key_count,
            self.inner.notes.len()
        )
    }
}

#[pymodule]
fn rhythm_open_exchange(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyRoxChart>()?;
    Ok(())
}
```

### WebAssembly Pattern (wasm-bindgen)

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmRoxChart {
    inner: RoxChart,
}

#[wasm_bindgen]
impl WasmRoxChart {
    #[wasm_bindgen(constructor)]
    pub fn new(key_count: u8) -> Result<WasmRoxChart, JsValue> {
        if key_count == 0 || key_count > 18 {
            return Err(JsValue::from_str("Invalid key count"));
        }
        
        Ok(Self {
            inner: RoxChart::new(key_count),
        })
    }
    
    #[wasm_bindgen(js_name = addNote)]
    pub fn add_note(&mut self, time_us: i64, column: u8) -> Result<(), JsValue> {
        if column >= self.inner.key_count {
            return Err(JsValue::from_str("Column out of range"));
        }
        
        self.inner.notes.push(Note::tap(time_us, column));
        Ok(())
    }
    
    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.inner.metadata.title.clone()
    }
    
    #[wasm_bindgen(js_name = toBytes)]
    pub fn to_bytes(&self) -> Result<Vec<u8>, JsValue> {
        encode_chart(&self.inner)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    
    #[wasm_bindgen(js_name = fromBytes)]
    pub fn from_bytes(data: &[u8]) -> Result<WasmRoxChart, JsValue> {
        let inner = decode_chart(data)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(Self { inner })
    }
}
```

### UniFFI Pattern (Multi-Target FFI)

**When to use**: When you need bindings for multiple languages (Kotlin, Swift, Python, Ruby) with minimal boilerplate.

**Why UniFFI**: Generates FFI bindings automatically from a UDL (Universal Definition Language) file or Rust macros, reducing manual C API work.

```rust
// lib.rs with UniFFI macros

use uniffi;

#[uniffi::export]
pub fn create_chart(key_count: u8) -> Result<RoxChart, RoxError> {
    if key_count == 0 || key_count > 18 {
        return Err(RoxError::InvalidKeyCount(key_count));
    }
    Ok(RoxChart::new(key_count))
}

#[derive(uniffi::Object)]
pub struct RoxChart {
    key_count: u8,
    notes: Vec<Note>,
    metadata: Metadata,
}

#[uniffi::export]
impl RoxChart {
    #[uniffi::constructor]
    pub fn new(key_count: u8) -> Self {
        Self {
            key_count,
            notes: Vec::new(),
            metadata: Metadata::default(),
        }
    }
    
    pub fn add_note(&mut self, time_us: i64, column: u8) -> Result<(), RoxError> {
        if column >= self.key_count {
            return Err(RoxError::InvalidColumn { 
                column, 
                max: self.key_count 
            });
        }
        
        self.notes.push(Note::tap(time_us, column));
        Ok(())
    }
    
    pub fn get_title(&self) -> String {
        self.metadata.title.clone()
    }
    
    pub fn set_title(&mut self, title: String) {
        self.metadata.title = title;
    }
    
    pub fn note_count(&self) -> u32 {
        self.notes.len() as u32
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
    #[error("Invalid key count: {0}")]
    InvalidKeyCount(u8),
    
    #[error("Invalid column {column} (max {max})")]
    InvalidColumn { column: u8, max: u8 },
    
    #[error("Decode failed: {0}")]
    DecodeFailed(String),
}

// Generate bindings
uniffi::setup_scaffolding!();
```

**Alternative: UDL File Approach**

```idl
// rox.udl

namespace rox {
    RoxChart create_chart(u8 key_count);
};

dictionary Metadata {
    string title;
    string artist;
    string creator;
    string difficulty_name;
    string audio_file;
};

[Error]
enum RoxError {
    "InvalidKeyCount",
    "InvalidColumn",
    "DecodeFailed",
};

interface RoxChart {
    constructor(u8 key_count);
    
    [Throws=RoxError]
    void add_note(i64 time_us, u8 column);
    
    string get_title();
    void set_title(string title);
    u32 note_count();
};
```

**Cargo.toml configuration**:

```toml
[dependencies]
uniffi = "0.25"

[build-dependencies]
uniffi = { version = "0.25", features = ["build"] }

[lib]
crate-type = ["cdylib", "staticlib"]
name = "rox_uniffi"
```

**build.rs**:

```rust
fn main() {
    uniffi::generate_scaffolding("src/rox.udl").unwrap();
}
```

**Generated bindings usage**:

```kotlin
// Kotlin (Android)
val chart = RoxChart(4u)
chart.addNote(1000000, 0u)
println("Title: ${chart.getTitle()}")
```

```swift
// Swift (iOS)
let chart = RoxChart(keyCount: 4)
try chart.addNote(timeUs: 1000000, column: 0)
print("Title: \(chart.getTitle())")
```

```python
# Python
from rox_uniffi import RoxChart

chart = RoxChart(4)
chart.add_note(1000000, 0)
print(f"Title: {chart.get_title()}")
```

**UniFFI vs Manual FFI Decision Tree**:

1. **Do you need multiple language targets?**
   - Yes, 3+ languages → Use UniFFI
   - No, 1-2 languages → Consider manual FFI

2. **Do you need fine-grained control over FFI?**
   - Yes → Manual C API
   - No → UniFFI

3. **Is your API complex with many types?**
   - Yes → UniFFI (less boilerplate)
   - No → Either works

4. **Target languages**:
   - Kotlin/Swift/Python/Ruby → UniFFI excellent choice
   - C/C++ only → Manual C API
   - C# → Manual C API or CsBindgen
   - JavaScript/WASM → wasm-bindgen

## Memory Management Patterns

### Pattern 1: Opaque Handles

```rust
/// Never expose internal structure
pub struct OpaqueHandle {
    _private: [u8; 0],
}

impl OpaqueHandle {
    pub fn from_chart(chart: RoxChart) -> *mut Self {
        Box::into_raw(Box::new(chart)) as *mut Self
    }
    
    pub unsafe fn as_chart<'a>(handle: *const Self) -> &'a RoxChart {
        &*(handle as *const RoxChart)
    }
    
    pub unsafe fn as_chart_mut<'a>(handle: *mut Self) -> &'a mut RoxChart {
        &mut *(handle as *mut RoxChart)
    }
}
```

### Pattern 2: Buffer Management

```rust
/// Caller provides buffer
#[no_mangle]
pub extern "C" fn rox_chart_encode(
    chart: *const RoxChart,
    buffer: *mut u8,
    buffer_len: usize,
    out_written: *mut usize,
) -> i32 {
    if chart.is_null() || buffer.is_null() || out_written.is_null() {
        return -1;
    }
    
    let chart = unsafe { &*chart };
    
    match encode_chart(chart) {
        Ok(data) => {
            if data.len() > buffer_len {
                return -2; // Buffer too small
            }
            
            unsafe {
                std::ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    buffer,
                    data.len(),
                );
                *out_written = data.len();
            }
            
            0
        }
        Err(_) => -3,
    }
}

/// Rust allocates, caller frees
#[no_mangle]
pub extern "C" fn rox_chart_encode_alloc(
    chart: *const RoxChart,
    out_buffer: *mut *mut u8,
    out_len: *mut usize,
) -> i32 {
    if chart.is_null() || out_buffer.is_null() || out_len.is_null() {
        return -1;
    }
    
    let chart = unsafe { &*chart };
    
    match encode_chart(chart) {
        Ok(data) => {
            let len = data.len();
            let ptr = data.as_ptr();
            
            unsafe {
                *out_buffer = ptr as *mut u8;
                *out_len = len;
            }
            
            std::mem::forget(data); // Caller owns now
            0
        }
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn rox_free_buffer(buffer: *mut u8, len: usize) {
    if !buffer.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(buffer, len, len);
        }
    }
}
```

## Testing FFI

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ffi_lifecycle() {
        // Create
        let chart = rox_chart_new(4);
        assert!(!chart.is_null());
        
        // Use
        let result = rox_chart_add_note(chart, 1000000, 0);
        assert_eq!(result, 0);
        
        // Destroy
        rox_chart_free(chart);
    }
    
    #[test]
    fn test_null_safety() {
        let result = rox_chart_add_note(std::ptr::null_mut(), 0, 0);
        assert_eq!(result, RoxError::NullPointer as i32);
    }
}
```

## Common Mistakes to Avoid

### ❌ Exposing Rust types directly

```rust
// BAD: Rust-specific types in FFI
#[no_mangle]
pub extern "C" fn bad_function(s: String) -> Vec<u8> {
    // String and Vec are not FFI-safe!
}
```

```rust
// GOOD: C-compatible types
#[no_mangle]
pub extern "C" fn good_function(
    s: *const c_char,
    out_buffer: *mut u8,
    buffer_len: usize,
) -> i32 {
    // Explicit, safe
}
```

### ❌ Panicking across FFI

```rust
// BAD
#[no_mangle]
pub extern "C" fn bad_decode(data: *const u8, len: usize) -> *mut RoxChart {
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let chart = decode_chart(slice).unwrap(); // PANIC!
    Box::into_raw(Box::new(chart))
}
```

```rust
// GOOD
#[no_mangle]
pub extern "C" fn good_decode(
    data: *const u8,
    len: usize,
    out_chart: *mut *mut RoxChart,
) -> i32 {
    std::panic::catch_unwind(|| {
        // Safe handling
    }).unwrap_or(-99)
}
```

### ❌ Memory leaks

```rust
// BAD: Leaks CString
#[no_mangle]
pub extern "C" fn get_title(chart: *const RoxChart) -> *const c_char {
    let chart = unsafe { &*chart };
    CString::new(chart.metadata.title.as_str())
        .unwrap()
        .into_raw() // Caller must free, but might not know!
}
```

```rust
// GOOD: Explicit ownership
#[no_mangle]
pub extern "C" fn get_title(
    chart: *const RoxChart,
    buffer: *mut c_char,
    buffer_len: usize,
) -> i32 {
    // Caller provides buffer, clear ownership
}
```

## Checklist

When creating FFI bindings:

- [ ] All exported structs use `#[repr(C)]`
- [ ] All FFI functions use `extern "C"`
- [ ] Null pointer checks on all inputs
- [ ] Panic handling with `catch_unwind`
- [ ] Clear ownership semantics documented
- [ ] Error codes defined and documented
- [ ] Memory management functions provided (create/destroy)
- [ ] Thread safety considered and documented
- [ ] Tests for null safety and error paths
- [ ] Example usage in target language

## References

- [Rust FFI Omnibus](http://jakegoulding.com/rust-ffi-omnibus/)
- [UniFFI User Guide](https://mozilla.github.io/uniffi-rs/)
- [PyO3 User Guide](https://pyo3.rs/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- User rule: `rust-strict-standards.md` (Error handling)

