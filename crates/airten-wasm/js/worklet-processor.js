/**
 * AirTen AudioWorklet Processor
 * 
 * This processor runs in the audio rendering thread and provides
 * real-time audio processing with minimal latency.
 */

class AirtenWorkletProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();
    
    this.processor = null;
    this.initialized = false;
    this.bypassed = false;
    
    // Handle messages from main thread
    this.port.onmessage = this.handleMessage.bind(this);
    
    // Request WASM module from main thread
    this.port.postMessage({ type: 'ready' });
  }

  handleMessage(event) {
    const { type, data } = event.data;
    
    switch (type) {
      case 'init':
        this.initProcessor(data);
        break;
      case 'bypass':
        this.bypassed = data.enabled;
        break;
      case 'reset':
        if (this.processor) {
          this.processor.reset();
        }
        break;
      case 'setParameter':
        this.setParameter(data.name, data.value);
        break;
    }
  }

  initProcessor(config) {
    try {
      // Import WASM module (passed from main thread)
      const { AudioProcessor, ProcessorOptions } = config.module;
      
      const options = new ProcessorOptions();
      options.sampleRate = sampleRate;
      options.frameSize = 128; // AudioWorklet quantum size
      options.channels = config.channels || 1;
      options.noiseSuppression = config.noiseSuppression ?? true;
      options.compression = config.compression ?? true;
      
      this.processor = new AudioProcessor(options);
      this.initialized = true;
      
      this.port.postMessage({ type: 'initialized' });
    } catch (error) {
      this.port.postMessage({ type: 'error', error: error.message });
    }
  }

  setParameter(name, value) {
    // Handle parameter changes
    // This could be extended to modify processor settings
  }

  process(inputs, outputs, parameters) {
    const input = inputs[0];
    const output = outputs[0];
    
    if (!input || !input[0] || !this.initialized || this.bypassed) {
      // Pass through or silence
      if (input && input[0] && output && output[0]) {
        for (let channel = 0; channel < output.length; channel++) {
          output[channel].set(input[channel] || new Float32Array(128));
        }
      }
      return true;
    }
    
    try {
      // Process each channel
      for (let channel = 0; channel < output.length; channel++) {
        const inputChannel = input[channel] || new Float32Array(128);
        const outputChannel = output[channel];
        
        // Copy input to output buffer
        outputChannel.set(inputChannel);
        
        // Process in-place
        this.processor.process(outputChannel);
      }
      
      // Send metrics to main thread periodically
      if (currentFrame % 4800 === 0) { // ~100ms at 48kHz
        this.port.postMessage({
          type: 'metrics',
          data: {
            envelopeLevel: this.processor.envelopeLevel,
          }
        });
      }
    } catch (error) {
      this.port.postMessage({ type: 'error', error: error.message });
    }
    
    return true;
  }
}

registerProcessor('airten-processor', AirtenWorkletProcessor);
