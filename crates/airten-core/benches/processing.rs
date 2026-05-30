use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use airten_core::{AudioProcessor, ProcessorConfig, Sample};
use airten_core::dsp::{BiquadFilter, Compressor, NoiseGate};

fn bench_processor(c: &mut Criterion) {
    let mut group = c.benchmark_group("AudioProcessor");
    
    for size in [128, 256, 512, 1024].iter() {
        group.bench_with_input(BenchmarkId::new("process", size), size, |b, &size| {
            let config = ProcessorConfig {
                frame_size: size,
                ..Default::default()
            };
            let mut processor = AudioProcessor::new(config);
            let mut samples = vec![0.5f32; size];
            
            b.iter(|| {
                processor.process(std::hint::black_box(&mut samples)).unwrap();
            });
        });
    }
    
    group.finish();
}

fn bench_biquad_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("BiquadFilter");
    
    for size in [128, 256, 512, 1024].iter() {
        group.bench_with_input(BenchmarkId::new("lowpass", size), size, |b, &size| {
            let mut filter = BiquadFilter::lowpass(48000.0, 1000.0, 0.707);
            let mut samples = vec![0.5f32; size];
            
            b.iter(|| {
                filter.process_block(std::hint::black_box(&mut samples));
            });
        });
    }
    
    group.finish();
}

fn bench_compressor(c: &mut Criterion) {
    let mut group = c.benchmark_group("Compressor");
    
    for size in [128, 256, 512, 1024].iter() {
        group.bench_with_input(BenchmarkId::new("process", size), size, |b, &size| {
            let mut compressor = Compressor::new(48000.0);
            let mut samples = vec![0.5f32; size];
            
            b.iter(|| {
                compressor.process_block(std::hint::black_box(&mut samples));
            });
        });
    }
    
    group.finish();
}

fn bench_noise_gate(c: &mut Criterion) {
    let mut group = c.benchmark_group("NoiseGate");
    
    for size in [128, 256, 512, 1024].iter() {
        group.bench_with_input(BenchmarkId::new("process", size), size, |b, &size| {
            let mut gate = NoiseGate::new(48000.0);
            let mut samples = vec![0.5f32; size];
            
            b.iter(|| {
                gate.process_block(std::hint::black_box(&mut samples));
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_processor,
    bench_biquad_filter,
    bench_compressor,
    bench_noise_gate,
);

criterion_main!(benches);
