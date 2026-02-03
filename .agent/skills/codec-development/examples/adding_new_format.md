# Codec Development Example - Adding a New Format

This example shows how to add support for a new rhythm game format to ROX.

## Example: Adding FNF (Friday Night Funkin') Support

### Step 1: Create Decoder Module

```rust
// src/codec/formats/fnf/decoder.rs

use crate::codec::error::CodecError;
use crate::model::{RoxChart, Note, TimingPoint, Metadata};
use serde::{Deserialize, Serialize};

const MAX_FNF_FILE_SIZE: usize = 50 * 1024 * 1024; // 50 MB

/// FNF chart JSON structure
#[derive(Debug, Deserialize)]
struct FnfChart {
    song: FnfSong,
}

#[derive(Debug, Deserialize)]
struct FnfSong {
    song: String,
    notes: Vec<FnfSection>,
    bpm: f64,
    #[serde(default)]
    speed: f64,
}

#[derive(Debug, Deserialize)]
struct FnfSection {
    #[serde(rename = "sectionNotes")]
    section_notes: Vec<FnfNote>,
    #[serde(rename = "mustHitSection")]
    must_hit_section: bool,
}

type FnfNote = (f64, u8, f64); // (time_ms, column, duration_ms)

/// Decodes FNF JSON format to RoxChart
pub fn decode_fnf(data: &[u8]) -> Result<RoxChart, CodecError> {
    // 1. Validate input
    validate_input(data)?;
    
    // 2. Parse JSON
    let fnf_chart: FnfChart = serde_json::from_slice(data)
        .map_err(|e| CodecError::ParseError {
            line: 0,
            message: format!("JSON parse error: {}", e),
        })?;
    
    // 3. Extract metadata
    let metadata = extract_metadata(&fnf_chart.song);
    
    // 4. Convert notes
    let notes = convert_notes(&fnf_chart.song)?;
    
    // 5. Create timing points
    let timing_points = vec![TimingPoint::bpm(0, fnf_chart.song.bpm)];
    
    // 6. Build chart
    let mut chart = RoxChart::new(4); // FNF is always 4K
    chart.metadata = metadata;
    chart.notes = notes;
    chart.timing_points = timing_points;
    
    // 7. Validate result
    validate_chart(&chart)?;
    
    Ok(chart)
}

fn validate_input(data: &[u8]) -> Result<(), CodecError> {
    if data.is_empty() {
        return Err(CodecError::EmptyInput);
    }
    
    if data.len() > MAX_FNF_FILE_SIZE {
        return Err(CodecError::FileTooLarge(data.len()));
    }
    
    Ok(())
}

fn extract_metadata(song: &FnfSong) -> Metadata {
    Metadata {
        title: song.song.clone(),
        artist: "Unknown".to_string(),
        creator: "Unknown".to_string(),
        difficulty_name: "Normal".to_string(),
        audio_file: format!("{}.ogg", song.song),
        ..Default::default()
    }
}

fn convert_notes(song: &FnfSong) -> Result<Vec<Note>, CodecError> {
    let mut notes = Vec::new();
    
    for section in &song.notes {
        for &(time_ms, column, duration_ms) in &section.section_notes {
            // Validate column
            if column >= 4 {
                return Err(CodecError::InvalidNote {
                    reason: format!("Invalid column {} (FNF is 4K)", column),
                });
            }
            
            // Convert time from milliseconds to microseconds
            let time_us = (time_ms * 1000.0) as i64;
            
            // Create note
            if duration_ms > 0.0 {
                let duration_us = (duration_ms * 1000.0) as u32;
                notes.push(Note::hold(time_us, duration_us, column));
            } else {
                notes.push(Note::tap(time_us, column));
            }
        }
    }
    
    // Sort by time
    notes.sort_by_key(|n| n.time);
    
    Ok(notes)
}

fn validate_chart(chart: &RoxChart) -> Result<(), CodecError> {
    if chart.key_count != 4 {
        return Err(CodecError::InvalidKeyCount(chart.key_count));
    }
    
    // Verify all notes are within valid columns
    for note in &chart.notes {
        if note.column >= 4 {
            return Err(CodecError::InvalidNote {
                reason: format!("Column {} out of range for 4K chart", note.column),
            });
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decode_fnf_basic() {
        let json = r#"{
            "song": {
                "song": "Test Song",
                "bpm": 120.0,
                "speed": 1.0,
                "notes": [
                    {
                        "sectionNotes": [
                            [0.0, 0, 0.0],
                            [500.0, 1, 0.0],
                            [1000.0, 2, 500.0]
                        ],
                        "mustHitSection": true
                    }
                ]
            }
        }"#;
        
        let chart = decode_fnf(json.as_bytes()).unwrap();
        
        assert_eq!(chart.key_count, 4);
        assert_eq!(chart.notes.len(), 3);
        assert_eq!(chart.metadata.title, "Test Song");
        assert_eq!(chart.timing_points[0].bpm, 120.0);
    }
    
    #[test]
    fn test_decode_fnf_empty() {
        let result = decode_fnf(b"");
        assert!(matches!(result, Err(CodecError::EmptyInput)));
    }
    
    #[test]
    fn test_decode_fnf_invalid_json() {
        let result = decode_fnf(b"not json");
        assert!(result.is_err());
    }
}
```

### Step 2: Create Encoder Module

```rust
// src/codec/formats/fnf/encoder.rs

use crate::codec::error::CodecError;
use crate::model::RoxChart;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct FnfChart {
    song: FnfSong,
}

#[derive(Debug, Serialize)]
struct FnfSong {
    song: String,
    notes: Vec<FnfSection>,
    bpm: f64,
    #[serde(rename = "needsVoices")]
    needs_voices: bool,
    speed: f64,
}

#[derive(Debug, Serialize)]
struct FnfSection {
    #[serde(rename = "sectionNotes")]
    section_notes: Vec<(f64, u8, f64)>,
    #[serde(rename = "mustHitSection")]
    must_hit_section: bool,
    #[serde(rename = "changeBPM")]
    change_bpm: bool,
    bpm: f64,
}

pub fn encode_fnf(chart: &RoxChart) -> Result<Vec<u8>, CodecError> {
    // 1. Validate chart
    validate_chart(chart)?;
    
    // 2. Get BPM
    let bpm = chart.timing_points
        .first()
        .map(|tp| tp.bpm)
        .unwrap_or(120.0);
    
    // 3. Convert notes to FNF format
    let section_notes: Vec<(f64, u8, f64)> = chart.notes
        .iter()
        .map(|note| {
            let time_ms = note.time as f64 / 1000.0;
            let duration_ms = note.duration as f64 / 1000.0;
            (time_ms, note.column, duration_ms)
        })
        .collect();
    
    // 4. Create FNF structure
    let fnf_chart = FnfChart {
        song: FnfSong {
            song: chart.metadata.title.clone(),
            bpm,
            needs_voices: false,
            speed: 1.0,
            notes: vec![FnfSection {
                section_notes,
                must_hit_section: true,
                change_bpm: false,
                bpm,
            }],
        },
    };
    
    // 5. Serialize to JSON
    serde_json::to_vec_pretty(&fnf_chart)
        .map_err(|e| CodecError::EncodeFailed(e.to_string()))
}

fn validate_chart(chart: &RoxChart) -> Result<(), CodecError> {
    if chart.key_count != 4 {
        return Err(CodecError::UnsupportedFeature(
            format!("FNF only supports 4K charts (got {}K)", chart.key_count)
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Note, TimingPoint, Metadata};
    
    #[test]
    fn test_encode_fnf() {
        let mut chart = RoxChart::new(4);
        chart.metadata.title = "Test Song".to_string();
        chart.timing_points.push(TimingPoint::bpm(0, 120.0));
        chart.notes.push(Note::tap(0, 0));
        chart.notes.push(Note::hold(500_000, 500_000, 1));
        
        let result = encode_fnf(&chart);
        assert!(result.is_ok());
        
        let json = String::from_utf8(result.unwrap()).unwrap();
        assert!(json.contains("Test Song"));
        assert!(json.contains("120"));
    }
}
```

### Step 3: Register Format

```rust
// src/codec/auto.rs

use crate::codec::formats::fnf::{decode_fnf, encode_fnf};

pub fn auto_decode(path: &Path) -> Result<RoxChart, CodecError> {
    let ext = path.extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| CodecError::UnsupportedFormat("No extension".into()))?;
    
    let data = std::fs::read(path)?;
    
    match ext {
        "rox" => decode_rox(&data),
        "osu" => decode_osu(&data),
        "sm" => decode_sm(&data),
        "qua" => decode_qua(&data),
        "json" => decode_fnf(&data), // Add FNF support
        _ => Err(CodecError::UnsupportedFormat(ext.to_string())),
    }
}
```

### Step 4: Add Roundtrip Test

```rust
// tests/codec_roundtrip.rs

#[test]
fn test_fnf_roundtrip() {
    let original = create_test_chart(4);
    
    // Encode
    let encoded = encode_fnf(&original).unwrap();
    
    // Decode
    let decoded = decode_fnf(&encoded).unwrap();
    
    // Verify
    assert_eq!(original.key_count, decoded.key_count);
    assert_eq!(original.notes.len(), decoded.notes.len());
    
    for (orig, dec) in original.notes.iter().zip(decoded.notes.iter()) {
        assert_eq!(orig.time, dec.time);
        assert_eq!(orig.column, dec.column);
        assert_eq!(orig.duration, dec.duration);
    }
}
```

## Key Takeaways

1. **Validate early**: Check input size, format, and structure before processing
2. **Handle format quirks**: FNF uses milliseconds, ROX uses microseconds
3. **Test thoroughly**: Roundtrip tests ensure fidelity
4. **Document assumptions**: FNF is always 4K, document this limitation
5. **Use domain errors**: Clear error types help debugging

## Common Pitfalls

- ❌ Forgetting time unit conversion (ms ↔ μs)
- ❌ Not validating column indices
- ❌ Assuming format supports all features
- ❌ Not sorting notes by time
- ❌ Skipping roundtrip tests
