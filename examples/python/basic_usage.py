"""Basic AirTen usage example."""

import numpy as np
import airten

def main():
    print("AirTen Python Example")
    print("=" * 50)
    print()

    # Create processor
    processor = airten.AudioProcessor(
        sample_rate=48000,
        frame_size=512,
        channels=1,
        noise_suppression=True,
        compression=True,
    )

    print(f"Sample Rate: {processor.sample_rate} Hz")
    print(f"Frame Size: {processor.frame_size} samples")
    print()

    # Generate test signal (sine wave at 440 Hz)
    duration = 1.0  # seconds
    t = np.linspace(0, duration, int(processor.sample_rate * duration), dtype=np.float32)
    frequency = 440.0  # A4 note
    audio = np.sin(2 * np.pi * frequency * t) * 0.5

    print(f"Processing {len(audio)} samples...")

    # Calculate input metrics
    input_rms = airten.calculate_rms(audio)
    input_peak = airten.find_peak(audio)
    
    print(f"Input RMS: {input_rms:.4f}")
    print(f"Input Peak: {input_peak:.4f}")
    print(f"Input Level: {airten.linear_to_db(input_rms):.2f} dB")

    # Process audio
    processed = processor.process(audio)

    # Calculate output metrics
    output_rms = airten.calculate_rms(processed)
    output_peak = airten.find_peak(processed)
    
    print(f"\nOutput RMS: {output_rms:.4f}")
    print(f"Output Peak: {output_peak:.4f}")
    print(f"Output Level: {airten.linear_to_db(output_rms):.2f} dB")
    print(f"Envelope Level: {processor.envelope_level:.4f}")

    print("\n✓ Processing complete!")

if __name__ == "__main__":
    main()
