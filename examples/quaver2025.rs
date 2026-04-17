use metron_rs::calculator::Calculator;
use metron_rs::clock_rate::ClockRate;
use metron_rs::quaver::quaver2025::difficulty::{Quaver2025, QuaverDifficultyContext};
use rhythm_open_exchange::auto_decode;

const RATES: &[u32] = &[70, 80, 90, 100, 110, 120, 130, 140, 150, 160];

fn main() {
    let chart = auto_decode("assets/test.osu").expect("Failed to decode test.osu");
    let calc = Quaver2025;

    println!("Quaver 2025 — {}", chart.metadata.title);
    println!("{:<8} {:>8}", "Rate", "Stars");
    println!("{}", "-".repeat(18));

    for &rate in RATES {
        let context = QuaverDifficultyContext {
            clock_rate: ClockRate::from_percentage(rate).unwrap(),
        };
        let d = calc.calculate_difficulty(&chart, &context).expect("Calculation failed");
        println!("{:<8} {:>8.2}", format!("{:.1}x", rate as f32 / 100.0), d.stars);
    }
}
