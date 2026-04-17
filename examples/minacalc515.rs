use metron_rs::calculator::Calculator;
use metron_rs::clock_rate::ClockRate;
use metron_rs::etterna::minacalc515::{MinaCalc515, MinaCalcDifficultyContext};
use rhythm_open_exchange::auto_decode;

const RATES: &[u32] = &[70, 80, 90, 100, 110, 120, 130, 140, 150, 160];

fn main() {
    let chart = auto_decode("assets/test.osu").expect("Failed to decode test.osu");
    let calc = MinaCalc515;

    println!("MinaCalc 515 — {}", chart.metadata.title);
    println!("{:<8} {:>8} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Rate", "Overall", "Stream", "Jumpstr.", "Handstr.", "Stamina", "Jack", "Chord", "Tech");
    println!("{}", "-".repeat(88));

    for &rate in RATES {
        let context = MinaCalcDifficultyContext {
            clock_rate: ClockRate::from_percentage(rate).unwrap(),
            ..Default::default()
        };
        let d = calc.calculate_difficulty(&chart, &context).expect("Calculation failed");
        println!("{:<8} {:>8.2} {:>10.2} {:>10.2} {:>10.2} {:>10.2} {:>10.2} {:>10.2} {:>10.2}",
            format!("{:.1}x", rate as f32 / 100.0),
            d.overall, d.stream, d.jumpstream, d.handstream,
            d.stamina, d.jackspeed, d.chordjack, d.technical);
    }
}
