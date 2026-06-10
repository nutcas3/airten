use airten_core::simd::{calculate_rms, find_peak, process_gain_scalar};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_gain_scalar(c: &mut Criterion) {
    let mut group = c.benchmark_group("Gain_Scalar");

    for size in [256, 512, 1024, 2048].iter() {
        group.bench_with_input(BenchmarkId::new("process", size), size, |b, &size| {
            let input = vec![0.5f32; size];
            let mut output = vec![0.0f32; size];

            b.iter(|| {
                process_gain_scalar(black_box(&input), black_box(&mut output), 0.8);
            });
        });
    }

    group.finish();
}

fn bench_rms(c: &mut Criterion) {
    let mut group = c.benchmark_group("RMS");

    for size in [256, 512, 1024, 2048].iter() {
        group.bench_with_input(BenchmarkId::new("calculate", size), size, |b, &size| {
            let samples = vec![0.5f32; size];

            b.iter(|| calculate_rms(black_box(&samples)));
        });
    }

    group.finish();
}

fn bench_peak(c: &mut Criterion) {
    let mut group = c.benchmark_group("Peak");

    for size in [256, 512, 1024, 2048].iter() {
        group.bench_with_input(BenchmarkId::new("find", size), size, |b, &size| {
            let samples = vec![0.5f32; size];

            b.iter(|| find_peak(black_box(&samples)));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_gain_scalar, bench_rms, bench_peak,);

criterion_main!(benches);
