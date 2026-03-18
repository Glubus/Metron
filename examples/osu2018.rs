use metron::calculator::Calculator;
use metron::clock_rate::ClockRate;
use metron::osu::osu2018::{Osu2018, Osu2018DifficultyContext};
use rhythm_open_exchange::auto_decode;

const RATES: &[u32] = &[70, 80, 90, 100, 110, 120, 130, 140, 150, 160];

fn main() {
    let chart = auto_decode("assets/test.osu").expect("Failed to decode test.osu");
    let calc = Osu2018;

    println!("osu!mania 2018 — {}", chart.metadata.title);
    println!("{:<8} {:>8}", "Rate", "Stars");
    println!("{}", "-".repeat(18));

    for &rate in RATES {
        let context = Osu2018DifficultyContext {
            clock_rate: Some(ClockRate::from_percentage(rate).unwrap()),
            ..Default::default()
        };
        let d = calc.calculate_difficulty(&chart, &context).expect("Calculation failed");
        println!("{:<8} {:>8.2}", format!("{:.1}x", rate as f32 / 100.0), d.stars);
    }
}
