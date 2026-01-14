//! Basic audio processing example

use airten_core::{AudioProcessor, ProcessorConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("AirTen Basic Processing Example");
    println!("================================\n");

    // Create processor with default configuration
    let config = ProcessorConfig {
        sample_rate: 48000,
        frame_size: 512,
        num_channels: 1,
        noise_suppression: true,
        compression: true,
        target_latency_ms: 10.0,
    };

    let mut processor = AudioProcessor::new(config);

    println!("Configuration:");
    println!("  Sample Rate: {} Hz", config.sample_rate);
    println!("  Frame Size: {} samples", config.frame_size);
    println!("  Channels: {}", config.num_channels);
    println!("  Noise Suppression: {}", config.noise_suppression);
    println!("  Compression: {}", config.compression);
    println!();

    // Generate test signal (sine wave)
    let frequency = 440.0; // A4 note
    let omega = 2.0 * std::f32::consts::PI * frequency / config.sample_rate as f32;
    let mut samples: Vec<f32> = (0..config.frame_size)
        .map(|i| (omega * i as f32).sin() * 0.5)
        .collect();

    println!("Processing {} samples...", samples.len());

    // Calculate input RMS
    let input_rms = (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt();
    println!("Input RMS: {:.4}", input_rms);

    // Process audio
    processor.process(&mut samples)?;

    // Calculate output RMS
    let output_rms = (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt();
    println!("Output RMS: {:.4}", output_rms);

    // Get envelope level
    let envelope = processor.envelope_level();
    println!("Envelope Level: {:.4}", envelope);

    println!("\n✓ Processing complete!");

    Ok(())
}
