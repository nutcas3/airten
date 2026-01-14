/**
 * Simple AirTen C example
 * 
 * Demonstrates basic audio processing using the C FFI.
 */

#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include "airten.h"

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

int main(void) {
    printf("AirTen C Example\n");
    printf("================\n\n");

    // Create processor configuration
    AirtenConfig config = {
        .sample_rate = 48000,
        .frame_size = 512,
        .num_channels = 1,
        .noise_suppression = 1,
        .compression = 1
    };

    // Create processor
    AirtenProcessor* processor = airten_processor_new(&config);
    if (!processor) {
        fprintf(stderr, "Failed to create processor\n");
        return 1;
    }

    printf("Configuration:\n");
    printf("  Sample Rate: %u Hz\n", config.sample_rate);
    printf("  Frame Size: %u samples\n", config.frame_size);
    printf("  Channels: %u\n", config.num_channels);
    printf("\n");

    // Generate test signal (sine wave at 440 Hz)
    float* samples = (float*)malloc(config.frame_size * sizeof(float));
    if (!samples) {
        fprintf(stderr, "Failed to allocate memory\n");
        airten_processor_free(processor);
        return 1;
    }

    float frequency = 440.0f; // A4 note
    float omega = 2.0f * M_PI * frequency / config.sample_rate;
    
    for (uint32_t i = 0; i < config.frame_size; i++) {
        samples[i] = sinf(omega * i) * 0.5f;
    }

    // Calculate input RMS
    float input_rms = airten_calculate_rms(samples, config.frame_size);
    printf("Input RMS: %.4f\n", input_rms);
    printf("Input Level: %.2f dB\n", airten_linear_to_db(input_rms));

    // Process audio
    AirtenError result = airten_process(processor, samples, config.frame_size);
    if (result != AIRTEN_OK) {
        fprintf(stderr, "Processing failed with error code: %d\n", result);
        free(samples);
        airten_processor_free(processor);
        return 1;
    }

    // Calculate output RMS
    float output_rms = airten_calculate_rms(samples, config.frame_size);
    printf("Output RMS: %.4f\n", output_rms);
    printf("Output Level: %.2f dB\n", airten_linear_to_db(output_rms));

    printf("\n✓ Processing complete!\n");

    // Cleanup
    free(samples);
    airten_processor_free(processor);

    return 0;
}
