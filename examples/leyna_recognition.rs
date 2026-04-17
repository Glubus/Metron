use metron_rs::custom::leyna_recognition;
use rox_formats::auto_decode;

fn main() {
    let chart = auto_decode("assets/test.osu").expect("Failed to decode test.osu");

    println!("Leyna Recognition — {}", chart.metadata.title);
    println!("Key count: {}", chart.key_count);

    let result = leyna_recognition::analyze(&chart);

    println!("Timeline entries: {}", result.timeline.entries.len());

    for entry in result.timeline.entries.iter().take(10) {
        println!(
            "  [{:>8}ms – {:>8}ms] {:?}",
            entry.start_time, entry.end_time, entry.pattern_type
        );
    }

    if result.timeline.entries.len() > 10 {
        println!("  ... ({} more)", result.timeline.entries.len() - 10);
    }
}
