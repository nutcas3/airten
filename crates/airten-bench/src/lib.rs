use airten_core::{AudioProcessor, ProcessorConfig, Sample};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub std_dev: Duration,
    pub throughput: f64,
}

impl BenchmarkResult {
    pub fn meets_latency_target(&self, target_ms: f64) -> bool {
        self.avg_duration.as_secs_f64() * 1000.0 < target_ms
    }

    pub fn latency_ms(&self) -> f64 {
        self.avg_duration.as_secs_f64() * 1000.0
    }
}

pub struct Benchmarker {
    warmup_iterations: usize,
    iterations: usize,
}

impl Benchmarker {
    pub fn new(iterations: usize) -> Self {
        Self {
            warmup_iterations: iterations / 10,
            iterations,
        }
    }

    pub fn with_warmup(mut self, warmup: usize) -> Self {
        self.warmup_iterations = warmup;
        self
    }

    pub fn run<F>(&self, name: &str, samples_per_iter: usize, mut f: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        // Warmup
        for _ in 0..self.warmup_iterations {
            f();
        }

        // Collect timings
        let mut durations = Vec::with_capacity(self.iterations);

        for _ in 0..self.iterations {
            let start = Instant::now();
            f();
            durations.push(start.elapsed());
        }

        // Calculate statistics
        let total: Duration = durations.iter().sum();
        let avg = total / self.iterations as u32;
        let min = *durations.iter().min().unwrap();
        let max = *durations.iter().max().unwrap();

        let avg_nanos = avg.as_nanos() as f64;
        let variance: f64 = durations
            .iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - avg_nanos;
                diff * diff
            })
            .sum::<f64>()
            / self.iterations as f64;
        let std_dev = Duration::from_nanos(variance.sqrt() as u64);

        let throughput = (samples_per_iter * self.iterations) as f64 / total.as_secs_f64();

        BenchmarkResult {
            name: name.to_string(),
            iterations: self.iterations,
            total_duration: total,
            avg_duration: avg,
            min_duration: min,
            max_duration: max,
            std_dev,
            throughput,
        }
    }
}

pub fn benchmark_processor(config: ProcessorConfig, iterations: usize) -> BenchmarkResult {
    let mut processor = AudioProcessor::new(config);
    let mut samples: Vec<Sample> = (0..config.frame_size)
        .map(|i| (i as f32 * 0.01).sin() * 0.5)
        .collect();

    let benchmarker = Benchmarker::new(iterations);
    benchmarker.run("AudioProcessor", config.frame_size, || {
        processor.process(&mut samples).unwrap();
    })
}

/// Latency test result
#[derive(Debug, Clone)]
pub struct LatencyTestResult {
    /// Frame size tested
    pub frame_size: usize,
    /// Sample rate
    pub sample_rate: u32,
    /// Processing latency in milliseconds
    pub processing_latency_ms: f64,
    /// Buffer latency in milliseconds
    pub buffer_latency_ms: f64,
    /// Total latency in milliseconds
    pub total_latency_ms: f64,
    /// Whether target was met
    pub target_met: bool,
}

/// Run latency tests for various frame sizes
pub fn run_latency_tests(sample_rate: u32, target_ms: f64) -> Vec<LatencyTestResult> {
    let frame_sizes = [64, 128, 256, 512, 1024];
    let mut results = Vec::new();

    for &frame_size in &frame_sizes {
        let config = ProcessorConfig {
            sample_rate,
            frame_size,
            num_channels: 1,
            noise_suppression: true,
            compression: true,
            target_latency_ms: target_ms as f32,
        };

        let bench_result = benchmark_processor(config, 1000);

        let processing_latency_ms = bench_result.latency_ms();
        let buffer_latency_ms = (frame_size as f64 / sample_rate as f64) * 1000.0;
        let total_latency_ms = processing_latency_ms + buffer_latency_ms;

        results.push(LatencyTestResult {
            frame_size,
            sample_rate,
            processing_latency_ms,
            buffer_latency_ms,
            total_latency_ms,
            target_met: total_latency_ms < target_ms,
        });
    }

    results
}

/// Generate a benchmark report
pub fn generate_report(results: &[BenchmarkResult]) -> String {
    let mut report = String::new();

    report.push_str("# AirTen Benchmark Report\n\n");
    report.push_str(
        "| Benchmark | Iterations | Avg (ms) | Min (ms) | Max (ms) | Throughput (M/s) |\n",
    );
    report.push_str(
        "|-----------|------------|----------|----------|----------|------------------|\n",
    );

    for result in results {
        report.push_str(&format!(
            "| {} | {} | {:.3} | {:.3} | {:.3} | {:.2} |\n",
            result.name,
            result.iterations,
            result.avg_duration.as_secs_f64() * 1000.0,
            result.min_duration.as_secs_f64() * 1000.0,
            result.max_duration.as_secs_f64() * 1000.0,
            result.throughput / 1_000_000.0,
        ));
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmarker() {
        let benchmarker = Benchmarker::new(100);
        let result = benchmarker.run("test", 512, || {
            std::hint::black_box(1 + 1);
        });

        assert_eq!(result.iterations, 100);
        assert!(result.avg_duration < Duration::from_millis(1));
    }

    #[test]
    fn test_processor_benchmark() {
        let config = ProcessorConfig::default();
        let result = benchmark_processor(config, 100);

        assert!(result.throughput > 0.0);
    }
}
