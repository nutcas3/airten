use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use airten_core::{AudioProcessor, ProcessorConfig, Sample};

mod commands;

#[derive(Parser)]
#[command(name = "airten")]
#[command(author, version, about = "AirTen Audio Processing CLI", long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Process an audio file
    Process {
        /// Input audio file
        #[arg(short, long)]
        input: PathBuf,

        /// Output audio file
        #[arg(short, long)]
        output: PathBuf,

        /// Disable noise suppression
        #[arg(long)]
        no_noise_suppression: bool,

        /// Disable compression
        #[arg(long)]
        no_compression: bool,

        /// Frame size for processing
        #[arg(long, default_value = "512")]
        frame_size: usize,
    },

    /// Show information about an audio file
    Info {
        /// Audio file to analyze
        #[arg(short, long)]
        file: PathBuf,
    },

    /// Run benchmarks
    Benchmark {
        /// Number of iterations
        #[arg(short, long, default_value = "1000")]
        iterations: usize,

        /// Frame size
        #[arg(long, default_value = "512")]
        frame_size: usize,
    },

    /// Show version information
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    match cli.command {
        Commands::Process {
            input,
            output,
            no_noise_suppression,
            no_compression,
            frame_size,
        } => {
            process_file(
                &input,
                &output,
                !no_noise_suppression,
                !no_compression,
                frame_size,
            )?;
        }
        Commands::Info { file } => {
            show_info(&file)?;
        }
        Commands::Benchmark {
            iterations,
            frame_size,
        } => {
            run_benchmark(iterations, frame_size)?;
        }
        Commands::Version => {
            println!("AirTen CLI v{}", env!("CARGO_PKG_VERSION"));
            println!("Core library v{}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}

fn process_file(
    input: &PathBuf,
    output: &PathBuf,
    noise_suppression: bool,
    compression: bool,
    frame_size: usize,
) -> Result<()> {
    println!(
        "{} Processing {}",
        style("→").cyan().bold(),
        style(input.display()).yellow()
    );

    // Read input file
    let mut reader = hound::WavReader::open(input)
        .with_context(|| format!("Failed to open input file: {}", input.display()))?;

    let spec = reader.spec();
    let sample_rate = spec.sample_rate;
    let channels = spec.channels as usize;

    info!(
        "Input: {} Hz, {} channels, {} bits",
        sample_rate, channels, spec.bits_per_sample
    );

    // Read all samples
    let samples: Vec<Sample> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().map(|s| s.unwrap()).collect(),
        hound::SampleFormat::Int => reader
            .samples::<i16>()
            .map(|s| s.unwrap() as f32 / 32768.0)
            .collect(),
    };

    let total_samples = samples.len();
    let total_frames = total_samples / channels / frame_size;

    // Create processor
    let config = ProcessorConfig {
        sample_rate,
        frame_size,
        num_channels: channels,
        noise_suppression,
        compression,
        target_latency_ms: 10.0,
    };
    let mut processor = AudioProcessor::new(config);

    // Setup progress bar
    let pb = ProgressBar::new(total_frames as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )?
            .progress_chars("#>-"),
    );

    // Process audio
    let mut output_samples = samples.clone();
    let start = Instant::now();

    for frame_idx in 0..total_frames {
        let start_idx = frame_idx * frame_size * channels;
        let end_idx = start_idx + frame_size * channels;

        if end_idx <= output_samples.len() {
            // Process each channel
            for ch in 0..channels {
                let mut channel_samples: Vec<Sample> = (0..frame_size)
                    .map(|i| output_samples[start_idx + i * channels + ch])
                    .collect();

                processor.process(&mut channel_samples)?;

                for (i, &sample) in channel_samples.iter().enumerate() {
                    output_samples[start_idx + i * channels + ch] = sample;
                }
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message("Processing complete");

    let duration = start.elapsed();
    let audio_duration = total_samples as f64 / sample_rate as f64 / channels as f64;
    let realtime_factor = audio_duration / duration.as_secs_f64();

    println!(
        "{} Processed {:.2}s of audio in {:.2}s ({:.1}x realtime)",
        style("✓").green().bold(),
        audio_duration,
        duration.as_secs_f64(),
        realtime_factor
    );

    // Write output file
    let out_spec = hound::WavSpec {
        channels: channels as u16,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(output, out_spec)
        .with_context(|| format!("Failed to create output file: {}", output.display()))?;

    for sample in &output_samples {
        writer.write_sample(*sample)?;
    }

    writer.finalize()?;

    println!(
        "{} Output written to {}",
        style("✓").green().bold(),
        style(output.display()).yellow()
    );

    Ok(())
}

fn show_info(file: &PathBuf) -> Result<()> {
    let reader = hound::WavReader::open(file)
        .with_context(|| format!("Failed to open file: {}", file.display()))?;

    let spec = reader.spec();
    let duration = reader.duration() as f64 / spec.sample_rate as f64;

    println!("{}", style("Audio File Information").bold().underlined());
    println!();
    println!("  File:        {}", style(file.display()).yellow());
    println!("  Format:      WAV");
    println!("  Sample Rate: {} Hz", spec.sample_rate);
    println!("  Channels:    {}", spec.channels);
    println!("  Bit Depth:   {}", spec.bits_per_sample);
    println!("  Duration:    {:.2}s", duration);
    println!(
        "  Samples:     {}",
        reader.duration() * spec.channels as u32
    );

    // Calculate statistics
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => {
            let mut r = hound::WavReader::open(file)?;
            r.samples::<f32>().map(|s| s.unwrap()).collect()
        }
        hound::SampleFormat::Int => {
            let mut r = hound::WavReader::open(file)?;
            r.samples::<i16>()
                .map(|s| s.unwrap() as f32 / 32768.0)
                .collect()
        }
    };

    let peak = samples.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
    let rms = (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt();
    let peak_db = if peak > 0.0 {
        20.0 * peak.log10()
    } else {
        -120.0
    };
    let rms_db = if rms > 0.0 {
        20.0 * rms.log10()
    } else {
        -120.0
    };

    println!();
    println!("{}", style("Audio Statistics").bold().underlined());
    println!();
    println!("  Peak Level:  {:.2} dB", peak_db);
    println!("  RMS Level:   {:.2} dB", rms_db);
    println!("  Crest Factor: {:.2} dB", peak_db - rms_db);

    Ok(())
}

fn run_benchmark(iterations: usize, frame_size: usize) -> Result<()> {
    println!(
        "{} Running benchmark ({} iterations, {} samples/frame)",
        style("→").cyan().bold(),
        iterations,
        frame_size
    );

    let config = ProcessorConfig {
        sample_rate: 48000,
        frame_size,
        num_channels: 1,
        noise_suppression: true,
        compression: true,
        target_latency_ms: 10.0,
    };
    let mut processor = AudioProcessor::new(config);

    // Generate test signal
    let mut samples: Vec<Sample> = (0..frame_size)
        .map(|i| (i as f32 * 0.01).sin() * 0.5)
        .collect();

    // Warmup
    for _ in 0..100 {
        processor.process(&mut samples)?;
    }

    // Benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        processor.process(&mut samples)?;
    }
    let duration = start.elapsed();

    let total_samples = iterations * frame_size;
    let samples_per_sec = total_samples as f64 / duration.as_secs_f64();
    let latency_us = duration.as_micros() as f64 / iterations as f64;
    let latency_ms = latency_us / 1000.0;

    println!();
    println!("{}", style("Benchmark Results").bold().underlined());
    println!();
    println!("  Iterations:     {}", iterations);
    println!("  Frame Size:     {} samples", frame_size);
    println!("  Total Duration: {:.2}ms", duration.as_secs_f64() * 1000.0);
    println!("  Avg Latency:    {:.3}ms", latency_ms);
    println!(
        "  Throughput:     {:.2}M samples/sec",
        samples_per_sec / 1_000_000.0
    );

    let realtime_factor = samples_per_sec / 48000.0;
    println!("  Realtime Factor: {:.1}x (at 48kHz)", realtime_factor);

    if latency_ms < 10.0 {
        println!(
            "\n  {} Latency target (<10ms) {}",
            style("✓").green().bold(),
            style("PASSED").green().bold()
        );
    } else {
        println!(
            "\n  {} Latency target (<10ms) {}",
            style("✗").red().bold(),
            style("FAILED").red().bold()
        );
    }

    Ok(())
}
