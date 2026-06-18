use metron_rs::calculator::Calculator;
use metron_rs::clock_rate::ClockRate;
use metron_rs::custom::daniel::{Daniel, DanielDifficultyContext};
use rox_formats::auto_decode;

const RATES: &[u32] = &[70, 80, 90, 100, 110, 120, 130, 140, 150, 160];

fn main() {
    let chart = auto_decode("assets/test.osu").expect("Failed to decode test.osu");
    let calc = Daniel;

    println!("Daniel Star Rating — {}", chart.metadata.title);
    println!("{:<8} {:>8} {:>8}", "Rate", "Stars", "Points");
    println!("{}", "-".repeat(28));

    for &rate in RATES {
        let context = DanielDifficultyContext {
            clock_rate: Some(ClockRate::from_percentage(rate).unwrap()),
            overall_difficulty: Some(8.0),
        };
        let difficulty = calc
            .calculate_difficulty(&chart, &context)
            .expect("Calculation failed");
        println!(
            "{:<8} {:>8.2} {:>8}",
            format!("{:.1}x", rate as f32 / 100.0),
            difficulty.stars,
            difficulty.graph.values.len()
        );
    }
}
