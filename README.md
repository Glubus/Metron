# Metron

A Rust library implementing difficulty and performance calculators for rhythm games, using [rhythm-open-exchange](https://github.com/rhythm-open-exchange) charts as input.

## Calculators

| Game | Calculator | Version | Notes |
|---|---|---|---|
| Etterna | MinaCalc | 515 | MSD (raw) + SSR (score-relative) |
| Quaver | Quaver | 2025 | Full strain model |
| osu!mania | osu!mania | 2018 | HitWindow + strain |
| osu!mania | osu!mania | 2016 | 2018 difficulty + 2016 PP formula |
| Interlude | Interlude | 2025 | Jack/trill strain model |
| osu!mania | Daniel | 2026 | Custom SR on `RoxChart`, with graph + factor curves |
| osu!mania | SunnyXXY | 2024 | Existing custom SR kept separate |

## Usage

All calculators implement the `Calculator` trait:

```rust
fn calculate_difficulty(&self, chart: &RoxChart, context: &Self::DifficultyContext) -> CalculatorResult<Self::Difficulty>;
fn calculate_performance(&self, chart: &RoxChart, difficulty: &Self::Difficulty, context: &Self::PerformanceContext) -> CalculatorResult<Self::Performance>;
```

### Example — Etterna MinaCalc 515

```rust
use metron::calculator::Calculator;
use metron::etterna::minacalc515::{MinaCalc515, MinaCalcDifficultyContext};
use rhythm_open_exchange::auto_decode;

let chart = auto_decode("my_chart.osu").unwrap();
let calc = MinaCalc515;
let context = MinaCalcDifficultyContext::default(); // 1.0x, MSD mode

let difficulty = calc.calculate_difficulty(&chart, &context).unwrap();
println!("Overall: {:.2}", difficulty.overall);
```

### Example — Quaver 2025

```rust
use metron::calculator::Calculator;
use metron::clock_rate::ClockRate;
use metron::quaver::quaver2025::difficulty::{Quaver2025, QuaverDifficultyContext};

let context = QuaverDifficultyContext {
    clock_rate: ClockRate::from_percentage(150).unwrap(), // 1.5x
};
let difficulty = Quaver2025.calculate_difficulty(&chart, &context).unwrap();
println!("Stars: {:.2}", difficulty.stars);
```

### Example — Daniel

```rust
use metron_rs::calculator::Calculator;
use metron_rs::custom::daniel::{Daniel, DanielDifficultyContext};
use rox_formats::auto_decode;

let chart = auto_decode("my_chart.rox").unwrap();
let difficulty = Daniel
    .calculate_difficulty(&chart, &DanielDifficultyContext::default())
    .unwrap();

println!("Stars: {:.2}", difficulty.stars);
println!("Graph points: {}", difficulty.graph.values.len());
let averages = difficulty.factor_averages();
println!("Pressing Intensity avg: {:.3}", averages.pressing_intensity);
```

Daniel is a standalone custom calculator.
It does not replace `SunnyXXY`, even though some internal implementation patterns are shared.

Input expectations:
- Daniel runs on `RoxChart`, so it works with any format you can decode into ROX.
- It is not tied to raw `.osu` parsing.
- The current implementation targets 1K to 10K charts.

Output shape:
- `stars`: final star rating
- `graph.times_ms` / `graph.values`: smoothed difficulty graph
- `factors`: raw factor curves for `pressing_intensity`, `unevenness`, `same_column_pressure`, and `cross_column_pressure`
- `factor_averages()`: trapezoidal averages of those curves

## Examples

Run any example against `assets/test.osu` across 10 rates (0.7x → 1.6x):

```sh
cargo run --example minacalc515
cargo run --example quaver2025
cargo run --example osu2018
cargo run --example osu2016
cargo run --example interlude2025
cargo run --example daniel
```

## Benchmarks

```sh
cargo bench
```

Runs 10 000 calculations per calculator in parallel (rayon) and reports throughput.
