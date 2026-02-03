---
name: codec-development
description: Patterns for developing format codecs (encoders/decoders) in the ROX project. Use when adding new format support, implementing parsers, or working with binary/text format conversion. Enforces validation, error handling, and format-specific best practices.
---

# Codec Development Skill

This skill provides guidance for developing robust format codecs (encoders and decoders) for the Rhythm Open Exchange project.

## When to use this skill

- Adding support for a new rhythm game format
- Implementing format-specific encoders or decoders
- Working with binary or text format parsing
- Converting between formats
- Debugging format compatibility issues
- Optimizing codec performance

## Core Principles

### 1. Validate All Input

**Rule**: Every decoder must validate input data before processing.

**Why**: Malformed files can cause panics, incorrect behavior, or security issues.

```rust
pub fn decode_osu(data: &[u8]) -> Result<RoxChart, CodecError> {
    // Validate size
    if data.is_empty() {
        return Err(CodecError::EmptyInput);
    }
    
    if data.len() > MAX_FILE_SIZE {
        return Err(CodecError::FileTooLarge(data.len()));
    }
    
    // Validate encoding
    let text = std::str::from_utf8(data)
        .map_err(|_| CodecError::InvalidEncoding)?;
    
    // Validate format signature
    if !text.starts_with("osu file format v") {
        return Err(CodecError::InvalidFormat);
    }
    
    // Proceed with parsing
    parse_osu_content(text)
}
```

### 2. Fail Fast with Clear Errors

**Rule**: Return specific errors as soon as invalid data is detected.

```rust
#[derive(Error, Debug)]
pub enum CodecError {
    #[error("Empty input data")]
    EmptyInput,
    
    #[error("File too large: {0} bytes (max {MAX_FILE_SIZE})")]
    FileTooLarge(usize),
    
    #[error("Invalid encoding (expected UTF-8)")]
    InvalidEncoding,
    
    #[error("Unsupported format version: {0}")]
    UnsupportedVersion(u32),
    
    #[error("Invalid key count: {0} (expected 1-18)")]
    InvalidKeyCount(u8),
    
    #[error("Parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },
}
```

### 3. Preserve Format Fidelity

**Rule**: Encoders should produce output that decoders can read back identically.

**Test Pattern**:

```rust
#[test]
fn test_roundtrip_fidelity() {
    let original = create_test_chart();
    
    // Encode
    let encoded = encode_osu(&original).unwrap();
    
    // Decode
    let decoded = decode_osu(&encoded).unwrap();
    
    // Should be identical
    assert_eq!(original.notes, decoded.notes);
    assert_eq!(original.timing_points, decoded.timing_points);
    assert_eq!(original.metadata.title, decoded.metadata.title);
}
```

### 4. Handle Format-Specific Quirks

**Rule**: Document and handle format-specific behaviors explicitly.

```rust
/// Decodes osu!mania format
///
/// # Format Quirks
/// - Timing points use milliseconds (we convert to microseconds)
/// - Column indices are reversed for 7K+ charts
/// - SV multipliers are relative to base SV
pub fn decode_osu(data: &[u8]) -> Result<RoxChart, CodecError> {
    // ...
    
    // Handle osu-specific column reversal for 7K+
    if key_count >= 7 {
        for note in &mut chart.notes {
            note.column = key_count - 1 - note.column;
        }
    }
    
    // Convert milliseconds to microseconds
    for note in &mut chart.notes {
        note.time *= 1000;
    }
    
    Ok(chart)
}
```

## Decoder Pattern

### Standard Decoder Structure

```rust
pub fn decode_format(data: &[u8]) -> Result<RoxChart, CodecError> {
    // 1. Validate input
    validate_input(data)?;
    
    // 2. Parse header/metadata
    let (metadata, rest) = parse_header(data)?;
    
    // 3. Parse timing information
    let (timing_points, rest) = parse_timing(rest)?;
    
    // 4. Parse notes
    let notes = parse_notes(rest)?;
    
    // 5. Construct chart
    let mut chart = RoxChart::new(metadata.key_count);
    chart.metadata = metadata;
    chart.timing_points = timing_points;
    chart.notes = notes;
    
    // 6. Validate result
    validate_chart(&chart)?;
    
    Ok(chart)
}

fn validate_input(data: &[u8]) -> Result<(), CodecError> {
    if data.is_empty() {
        return Err(CodecError::EmptyInput);
    }
    
    if data.len() > MAX_FILE_SIZE {
        return Err(CodecError::FileTooLarge(data.len()));
    }
    
    Ok(())
}

fn validate_chart(chart: &RoxChart) -> Result<(), CodecError> {
    if chart.key_count == 0 || chart.key_count > 18 {
        return Err(CodecError::InvalidKeyCount(chart.key_count));
    }
    
    // Verify all notes are within valid columns
    for note in &chart.notes {
        if note.column >= chart.key_count {
            return Err(CodecError::InvalidNote {
                reason: format!(
                    "Column {} out of range for {}K chart",
                    note.column, chart.key_count
                ),
            });
        }
    }
    
    Ok(())
}
```

### Text Format Parsing

```rust
pub fn parse_osu_content(text: &str) -> Result<RoxChart, CodecError> {
    let mut lines = text.lines().enumerate();
    let mut current_section = None;
    
    let mut metadata = Metadata::default();
    let mut timing_points = Vec::new();
    let mut notes = Vec::new();
    
    for (line_num, line) in lines {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        
        // Section headers
        if line.starts_with('[') && line.ends_with(']') {
            current_section = Some(&line[1..line.len()-1]);
            continue;
        }
        
        // Parse based on current section
        match current_section {
            Some("General") => parse_general_line(line, &mut metadata)?,
            Some("Metadata") => parse_metadata_line(line, &mut metadata)?,
            Some("TimingPoints") => {
                timing_points.push(parse_timing_point(line, line_num)?);
            }
            Some("HitObjects") => {
                notes.push(parse_hit_object(line, line_num)?);
            }
            _ => {} // Ignore unknown sections
        }
    }
    
    // Build chart
    let mut chart = RoxChart::new(metadata.key_count);
    chart.metadata = metadata;
    chart.timing_points = timing_points;
    chart.notes = notes;
    
    Ok(chart)
}

fn parse_timing_point(line: &str, line_num: usize) -> Result<TimingPoint, CodecError> {
    let parts: Vec<&str> = line.split(',').collect();
    
    if parts.len() < 2 {
        return Err(CodecError::ParseError {
            line: line_num,
            message: "Invalid timing point format".into(),
        });
    }
    
    let time = parts[0].parse::<f64>()
        .map_err(|_| CodecError::ParseError {
            line: line_num,
            message: "Invalid time value".into(),
        })?;
    
    let beat_length = parts[1].parse::<f64>()
        .map_err(|_| CodecError::ParseError {
            line: line_num,
            message: "Invalid beat length".into(),
        })?;
    
    // Convert to microseconds
    let time_us = (time * 1000.0) as i64;
    
    // Calculate BPM
    let bpm = if beat_length > 0.0 {
        60_000.0 / beat_length
    } else {
        return Err(CodecError::ParseError {
            line: line_num,
            message: "Invalid beat length (must be > 0)".into(),
        });
    };
    
    Ok(TimingPoint::bpm(time_us, bpm))
}
```

### Binary Format Parsing

```rust
use std::io::{Read, Cursor};
use byteorder::{LittleEndian, ReadBytesExt};

pub fn decode_binary_format(data: &[u8]) -> Result<RoxChart, CodecError> {
    let mut cursor = Cursor::new(data);
    
    // Read magic number
    let magic = cursor.read_u32::<LittleEndian>()?;
    if magic != FORMAT_MAGIC {
        return Err(CodecError::InvalidFormat);
    }
    
    // Read version
    let version = cursor.read_u16::<LittleEndian>()?;
    if version > SUPPORTED_VERSION {
        return Err(CodecError::UnsupportedVersion(version as u32));
    }
    
    // Read key count
    let key_count = cursor.read_u8()?;
    if key_count == 0 || key_count > 18 {
        return Err(CodecError::InvalidKeyCount(key_count));
    }
    
    // Read note count
    let note_count = cursor.read_u32::<LittleEndian>()? as usize;
    if note_count > MAX_NOTES {
        return Err(CodecError::TooManyNotes(note_count));
    }
    
    // Read notes
    let mut notes = Vec::with_capacity(note_count);
    for _ in 0..note_count {
        let time = cursor.read_i64::<LittleEndian>()?;
        let column = cursor.read_u8()?;
        let note_type = cursor.read_u8()?;
        let duration = cursor.read_u32::<LittleEndian>()?;
        
        notes.push(Note {
            time,
            column,
            note_type,
            duration,
        });
    }
    
    let mut chart = RoxChart::new(key_count);
    chart.notes = notes;
    
    Ok(chart)
}
```

## Encoder Pattern

### Standard Encoder Structure

```rust
pub fn encode_format(chart: &RoxChart) -> Result<Vec<u8>, CodecError> {
    // 1. Validate chart
    validate_chart(chart)?;
    
    // 2. Estimate output size
    let estimated_size = estimate_output_size(chart);
    let mut output = Vec::with_capacity(estimated_size);
    
    // 3. Write header
    write_header(&mut output, chart)?;
    
    // 4. Write timing points
    write_timing_points(&mut output, &chart.timing_points)?;
    
    // 5. Write notes
    write_notes(&mut output, &chart.notes)?;
    
    Ok(output)
}

fn estimate_output_size(chart: &RoxChart) -> usize {
    // Rough estimate to minimize reallocations
    let header_size = 1024;
    let note_size = 50; // Average bytes per note
    let timing_size = 30; // Average bytes per timing point
    
    header_size
        + (chart.notes.len() * note_size)
        + (chart.timing_points.len() * timing_size)
}
```

### Text Format Encoding

```rust
use std::fmt::Write;

pub fn encode_osu(chart: &RoxChart) -> Result<Vec<u8>, CodecError> {
    let mut output = String::with_capacity(estimate_output_size(chart));
    
    // Header
    writeln!(output, "osu file format v14")?;
    writeln!(output)?;
    
    // General section
    writeln!(output, "[General]")?;
    writeln!(output, "AudioFilename: {}", chart.metadata.audio_file)?;
    writeln!(output, "Mode: 3")?; // mania
    writeln!(output)?;
    
    // Metadata section
    writeln!(output, "[Metadata]")?;
    writeln!(output, "Title:{}", chart.metadata.title)?;
    writeln!(output, "Artist:{}", chart.metadata.artist)?;
    writeln!(output, "Creator:{}", chart.metadata.creator)?;
    writeln!(output, "Version:{}", chart.metadata.difficulty_name)?;
    writeln!(output)?;
    
    // Difficulty section
    writeln!(output, "[Difficulty]")?;
    writeln!(output, "CircleSize:{}", chart.key_count)?;
    writeln!(output)?;
    
    // Timing points
    writeln!(output, "[TimingPoints]")?;
    for tp in &chart.timing_points {
        let time_ms = tp.time as f64 / 1000.0;
        let beat_length = 60_000.0 / tp.bpm;
        writeln!(output, "{},{},4,0,0,100,1,0", time_ms, beat_length)?;
    }
    writeln!(output)?;
    
    // Hit objects
    writeln!(output, "[HitObjects]")?;
    for note in &chart.notes {
        let time_ms = note.time / 1000;
        let x = (note.column as f32 / chart.key_count as f32 * 512.0) as i32;
        
        if note.duration > 0 {
            // Hold note
            let end_time = (note.time + note.duration as i64) / 1000;
            writeln!(output, "{},192,{},128,0,{}:0:0:0:0:", x, time_ms, end_time)?;
        } else {
            // Tap note
            writeln!(output, "{},192,{},1,0,0:0:0:0:", x, time_ms)?;
        }
    }
    
    Ok(output.into_bytes())
}
```

## Testing Codecs

### Roundtrip Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_roundtrip() {
        let original = RoxChart {
            key_count: 4,
            notes: vec![
                Note::tap(1_000_000, 0),
                Note::hold(2_000_000, 500_000, 1),
            ],
            timing_points: vec![
                TimingPoint::bpm(0, 180.0),
            ],
            ..Default::default()
        };
        
        let encoded = encode_osu(&original).unwrap();
        let decoded = decode_osu(&encoded).unwrap();
        
        assert_eq!(original.key_count, decoded.key_count);
        assert_eq!(original.notes.len(), decoded.notes.len());
    }
    
    #[test]
    fn test_decode_real_file() {
        let data = std::fs::read("assets/test.osu").unwrap();
        let chart = decode_osu(&data).unwrap();
        
        assert!(chart.key_count > 0);
        assert!(!chart.notes.is_empty());
    }
}
```

### Fuzzing

```rust
// fuzz/fuzz_targets/decode_osu.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Should never panic, only return errors
    let _ = decode_osu(data);
});
```

## Common Mistakes to Avoid

### ❌ No input validation

```rust
// BAD
pub fn decode(data: &[u8]) -> RoxChart {
    let text = std::str::from_utf8(data).unwrap(); // Can panic!
    parse(text)
}
```

### ❌ Lossy conversions

```rust
// BAD: Loses precision
let time_us = (time_ms * 1000) as i64; // Truncates!

// GOOD: Preserves precision
let time_us = (time_ms * 1000.0).round() as i64;
```

### ❌ Ignoring format quirks

```rust
// BAD: Assumes all formats use same time units
for note in notes {
    chart.notes.push(note); // Wrong time scale!
}

// GOOD: Convert to standard units
for note in notes {
    let mut converted = note;
    converted.time *= 1000; // ms -> μs
    chart.notes.push(converted);
}
```

## Checklist

When implementing a codec:

- [ ] Input validation (size, encoding, format signature)
- [ ] Clear, specific error types
- [ ] Roundtrip test (encode → decode → compare)
- [ ] Test with real format files
- [ ] Document format-specific quirks
- [ ] Handle edge cases (empty files, huge files, malformed data)
- [ ] Optimize for common case (pre-allocate buffers)
- [ ] Add fuzzing target
- [ ] Update `auto_decode` / `auto_encode` registration

## References

- User rule: `rule-input.md` (Input Validation)
- User rule: `rust-strict-standards.md` (Error Handling)
- Skill: `rust-error-handling`
- [nom parser combinators](https://github.com/rust-bakery/nom)
