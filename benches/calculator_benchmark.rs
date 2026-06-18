use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use criterion::{Criterion, criterion_group, criterion_main};
use metron_rs::calculator::Calculator;
use metron_rs::custom::daniel::{Daniel, DanielDifficultyContext};
use metron_rs::custom::sunnyxxy::{SunnyXXY, SunnyxxyDifficultyContext};
use metron_rs::etterna::minacalc515::{MinaCalc515, MinaCalcDifficultyContext};
use metron_rs::interlude::interlude2025::{Interlude2025, Interlude2025DifficultyContext};
use metron_rs::osu::osu2016::Osu2016;
use metron_rs::osu::osu2018::{Osu2018, Osu2018DifficultyContext};
use metron_rs::quaver::quaver2025::difficulty::{Quaver2025, QuaverDifficultyContext};
use rox_formats::auto_decode;
use std::hint::black_box;
use std::time::Duration;

use criterion::Throughput;
use rayon::prelude::*;

const BATCH_SIZE: u64 = 10000;
const SUNNYXXY_BATCH: u64 = 100;
const DANIEL_BATCH: u64 = 100;

fn benchmark_calculators(c: &mut Criterion) {
    let chart = auto_decode("assets/test.osu").expect("Failed to decode test.osu");

    let mut group = c.benchmark_group("calculators");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));
    group.throughput(Throughput::Elements(BATCH_SIZE));

    group.bench_function("osu2016", |b| {
        let calc = Osu2016;
        let context = Osu2018DifficultyContext::default();
        b.iter(|| {
            (0..BATCH_SIZE).into_par_iter().for_each(|_| {
                calc.calculate_difficulty(black_box(&chart), black_box(&context))
                    .expect("Calculation failed");
            });
        })
    });

    group.bench_function("osu2018", |b| {
        let calc = Osu2018;
        let context = Osu2018DifficultyContext::default();
        b.iter(|| {
            (0..BATCH_SIZE).into_par_iter().for_each(|_| {
                calc.calculate_difficulty(black_box(&chart), black_box(&context))
                    .expect("Calculation failed");
            });
        })
    });

    group.bench_function("interlude2025", |b| {
        let calc = Interlude2025;
        let context = Interlude2025DifficultyContext::default();
        b.iter(|| {
            (0..BATCH_SIZE).into_par_iter().for_each(|_| {
                calc.calculate_difficulty(black_box(&chart), black_box(&context))
                    .expect("Calculation failed");
            });
        })
    });

    group.bench_function("quaver2025", |b| {
        let calc = Quaver2025;
        let context = QuaverDifficultyContext::default();
        b.iter(|| {
            (0..BATCH_SIZE).into_par_iter().for_each(|_| {
                calc.calculate_difficulty(black_box(&chart), black_box(&context))
                    .expect("Calculation failed");
            });
        })
    });

    group.bench_function("minacalc515", |b| {
        let calc = MinaCalc515;
        let context = MinaCalcDifficultyContext::default();
        b.iter(|| {
            (0..BATCH_SIZE).into_par_iter().for_each(|_| {
                calc.calculate_difficulty(black_box(&chart), black_box(&context))
                    .expect("Calculation failed");
            });
        })
    });

    group.finish();
}

fn benchmark_sunnyxxy(c: &mut Criterion) {
    let chart = auto_decode("assets/test.osu").expect("Failed to decode test.osu");
    let calc = SunnyXXY;
    let context = SunnyxxyDifficultyContext::default();

    let mut group = c.benchmark_group("sunnyxxy");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));
    group.throughput(Throughput::Elements(SUNNYXXY_BATCH));

    group.bench_function("difficulty", |b| {
        b.iter(|| {
            (0..SUNNYXXY_BATCH).into_par_iter().for_each(|_| {
                calc.calculate_difficulty(black_box(&chart), black_box(&context))
                    .expect("Calculation failed");
            });
        })
    });

    group.finish();
}

fn benchmark_daniel(c: &mut Criterion) {
    let chart = auto_decode("assets/test.osu").expect("Failed to decode test.osu");
    let calc = Daniel;
    let context = DanielDifficultyContext::default();

    let mut group = c.benchmark_group("daniel");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));
    group.throughput(Throughput::Elements(DANIEL_BATCH));

    group.bench_function("difficulty", |b| {
        b.iter(|| {
            (0..DANIEL_BATCH).into_par_iter().for_each(|_| {
                calc.calculate_difficulty(black_box(&chart), black_box(&context))
                    .expect("Calculation failed");
            });
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_calculators,
    benchmark_sunnyxxy,
    benchmark_daniel
);
criterion_main!(benches);
