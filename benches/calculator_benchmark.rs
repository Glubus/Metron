use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use metron::calculator::Calculator;
use metron::interlude::interlude2025::{Interlude2025, Interlude2025DifficultyContext};
use metron::osu::osu2016::Osu2016;
use metron::osu::osu2018::{Osu2018, Osu2018DifficultyContext};
use metron::quaver::quaver2025::difficulty::{Quaver2025, QuaverDifficultyContext};
use rhythm_open_exchange::auto_decode;
use std::time::Duration;

use rayon::prelude::*;
use criterion::Throughput;

const BATCH_SIZE: u64 = 10000;

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

    group.finish();
}

criterion_group!(benches, benchmark_calculators);
criterion_main!(benches);
