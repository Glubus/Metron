use metron_rs::calculator::Calculator;
use metron_rs::etterna::minacalc527::{MinaCalc527, MinaCalcDifficultyContext};
use metron_rs::etterna::rows::{Rows, rebased_times};
use rox::RoxChart;
use rox::formats::auto_decode;
use rox::formats::osu::{OsuDecodeOptions, OsuDecoder};

fn load(path: &str) -> Result<RoxChart, String> {
    match auto_decode(path) {
        Ok(c) => Ok(c),
        Err(e) => {
            if path.ends_with(".osu") {
                let data = std::fs::read(path).map_err(|e| e.to_string())?;
                let opts = OsuDecodeOptions {
                    re_arrange_bpm: true,
                };
                OsuDecoder::decode_with_options(&data, &opts)
                    .map_err(|e2| format!("{e} / rearranged: {e2}"))
            } else {
                Err(e.to_string())
            }
        }
    }
}

fn main() {
    println!("file\tsame_times\texact_overall\tquant_overall\tdelta_overall\tmax_field_delta");
    for path in std::env::args().skip(1) {
        let name = path.rsplit('/').next().unwrap_or(&path).to_string();
        let chart = match load(&path) {
            Ok(c) => c,
            Err(e) => {
                println!("{name}\tERR\t{e}");
                continue;
            }
        };
        let same = rebased_times(&chart, Rows::Exact) == rebased_times(&chart, Rows::Quantized);
        let exact_ctx = MinaCalcDifficultyContext {
            rows: Rows::Exact,
            ..Default::default()
        };
        let quant_ctx = MinaCalcDifficultyContext {
            rows: Rows::Quantized,
            ..Default::default()
        };
        match (
            MinaCalc527.calculate_difficulty(&chart, &exact_ctx),
            MinaCalc527.calculate_difficulty(&chart, &quant_ctx),
        ) {
            (Ok(e), Ok(q)) => {
                let fields = [
                    (e.overall, q.overall),
                    (e.stream, q.stream),
                    (e.jumpstream, q.jumpstream),
                    (e.handstream, q.handstream),
                    (e.stamina, q.stamina),
                    (e.jackspeed, q.jackspeed),
                    (e.chordjack, q.chordjack),
                    (e.technical, q.technical),
                ];
                let max_field = fields
                    .iter()
                    .map(|(a, b)| (a - b).abs())
                    .fold(0.0_f32, f32::max);
                println!(
                    "{}\t{}\t{:.6}\t{:.6}\t{:.6}\t{:.6}",
                    name,
                    same,
                    e.overall,
                    q.overall,
                    (e.overall - q.overall).abs(),
                    max_field
                );
            }
            (e, q) => println!(
                "{name}\tERR\texact={:?}\tquant={:?}",
                e.err().map(|x| x.to_string()),
                q.err().map(|x| x.to_string())
            ),
        }
    }
}
