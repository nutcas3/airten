use airten_core::{AudioProcessor, ProcessorConfig};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn bench_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("Latency");
    group.significance_level(0.1).sample_size(1000);

    for frame_size in [64, 128, 256, 512].iter() {
        group.bench_with_input(
            BenchmarkId::new("target_10ms", frame_size),
            frame_size,
            |b, &size| {
                let config = ProcessorConfig {
                    sample_rate: 48000,
                    frame_size: size,
                    ..Default::default()
                };
                let mut processor = AudioProcessor::new(config);
                let mut samples = vec![0.5f32; size];

                b.iter(|| {
                    processor
                        .process(std::hint::black_box(&mut samples))
                        .unwrap();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_latency);
criterion_main!(benches);
