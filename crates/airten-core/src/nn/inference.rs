use crate::Sample;
use crate::error::{Error, Result};
use crate::nn::layer::Layer;

pub const MAX_LAYERS: usize = 16;
pub const MAX_LAYER_SIZE: usize = 512;

pub struct NeuralNetwork {
    layers: [Option<&'static Layer>; MAX_LAYERS],
    num_layers: usize,
    scratch_a: [Sample; MAX_LAYER_SIZE],
    scratch_b: [Sample; MAX_LAYER_SIZE],
}

impl NeuralNetwork {
    pub const fn new() -> Self {
        Self {
            layers: [None; MAX_LAYERS],
            num_layers: 0,
            scratch_a: [0.0; MAX_LAYER_SIZE],
            scratch_b: [0.0; MAX_LAYER_SIZE],
        }
    }

    pub fn add_layer(&mut self, layer: &'static Layer) -> Result<()> {
        if self.num_layers >= MAX_LAYERS {
            return Err(Error::BufferTooLarge);
        }
        
        // Validate layer connectivity
        if self.num_layers > 0 {
            if let Some(prev) = self.layers[self.num_layers - 1] {
                if prev.output_size() != layer.input_size() {
                    return Err(Error::InvalidModelFormat);
                }
            }
        }
        
        self.layers[self.num_layers] = Some(layer);
        self.num_layers += 1;
        Ok(())
    }

    #[inline]
    pub fn num_layers(&self) -> usize {
        self.num_layers
    }

    pub fn input_size(&self) -> Option<usize> {
        self.layers[0].map(|l| l.input_size())
    }

    pub fn output_size(&self) -> Option<usize> {
        if self.num_layers > 0 {
            self.layers[self.num_layers - 1].map(|l| l.output_size())
        } else {
            None
        }
    }

    pub fn forward(&mut self, input: &[Sample], output: &mut [Sample]) -> Result<()> {
        if self.num_layers == 0 {
            return Err(Error::ModelNotLoaded);
        }

        let first_layer = self.layers[0].ok_or(Error::ModelNotLoaded)?;
        if input.len() != first_layer.input_size() {
            return Err(Error::InvalidBufferSize);
        }

        let last_layer = self.layers[self.num_layers - 1].ok_or(Error::ModelNotLoaded)?;
        if output.len() != last_layer.output_size() {
            return Err(Error::InvalidBufferSize);
        }

        // Single layer case
        if self.num_layers == 1 {
            first_layer.forward(input, output);
            return Ok(());
        }

        // Multi-layer case: alternate between scratch buffers
        let mut use_a = true;
        
        // First layer
        first_layer.forward(input, &mut self.scratch_a[..first_layer.output_size()]);

        // Middle layers
        for i in 1..self.num_layers - 1 {
            let layer = self.layers[i].ok_or(Error::ModelNotLoaded)?;
            let in_size = layer.input_size();
            let out_size = layer.output_size();
            
            if use_a {
                layer.forward(
                    &self.scratch_a[..in_size],
                    &mut self.scratch_b[..out_size],
                );
            } else {
                layer.forward(
                    &self.scratch_b[..in_size],
                    &mut self.scratch_a[..out_size],
                );
            }
            use_a = !use_a;
        }

        // Last layer
        let in_size = last_layer.input_size();
        if use_a {
            last_layer.forward(&self.scratch_a[..in_size], output);
        } else {
            last_layer.forward(&self.scratch_b[..in_size], output);
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.layers = [None; MAX_LAYERS];
        self.num_layers = 0;
    }
}

impl Default for NeuralNetwork {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DenoiserNetwork {
    network: NeuralNetwork,
    frame_size: usize,
}

impl DenoiserNetwork {
    pub fn new(frame_size: usize) -> Self {
        Self {
            network: NeuralNetwork::new(),
            frame_size,
        }
    }

    pub fn process(&mut self, input: &[Sample], output: &mut [Sample]) -> Result<()> {
        if input.len() != self.frame_size || output.len() != self.frame_size {
            return Err(Error::InvalidBufferSize);
        }

        // If no model loaded, pass through
        if self.network.num_layers() == 0 {
            output.copy_from_slice(input);
            return Ok(());
        }

        self.network.forward(input, output)
    }

    pub fn network_mut(&mut self) -> &mut NeuralNetwork {
        &mut self.network
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nn::activation::{Activation, ActivationType};

    static LAYER1_WEIGHTS: [Sample; 4] = [1.0, 0.0, 0.0, 1.0];
    static LAYER1_BIASES: [Sample; 2] = [0.0, 0.0];
    static LAYER2_WEIGHTS: [Sample; 4] = [0.5, 0.5, 0.5, 0.5];
    static LAYER2_BIASES: [Sample; 2] = [0.0, 0.0];

    static LAYER1: Layer = Layer::new(&LAYER1_WEIGHTS, &LAYER1_BIASES, 2, 2);
    static LAYER2: Layer = Layer::new(&LAYER2_WEIGHTS, &LAYER2_BIASES, 2, 2);

    #[test]
    fn test_network_creation() {
        let network = NeuralNetwork::new();
        assert_eq!(network.num_layers(), 0);
    }

    #[test]
    fn test_add_layer() {
        let mut network = NeuralNetwork::new();
        assert!(network.add_layer(&LAYER1).is_ok());
        assert_eq!(network.num_layers(), 1);
        assert_eq!(network.input_size(), Some(2));
        assert_eq!(network.output_size(), Some(2));
    }

    #[test]
    fn test_forward_single_layer() {
        let mut network = NeuralNetwork::new();
        network.add_layer(&LAYER1).unwrap();

        let input = [1.0, 2.0];
        let mut output = [0.0; 2];

        network.forward(&input, &mut output).unwrap();

        assert!((output[0] - 1.0).abs() < 0.001);
        assert!((output[1] - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_forward_multi_layer() {
        let mut network = NeuralNetwork::new();
        network.add_layer(&LAYER1).unwrap();
        network.add_layer(&LAYER2).unwrap();

        let input = [1.0, 2.0];
        let mut output = [0.0; 2];

        network.forward(&input, &mut output).unwrap();

        assert!((output[0] - 1.5).abs() < 0.001);
        assert!((output[1] - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_empty_network_error() {
        let mut network = NeuralNetwork::new();
        let input = [1.0, 2.0];
        let mut output = [0.0; 2];

        let result = network.forward(&input, &mut output);
        assert!(matches!(result, Err(Error::ModelNotLoaded)));
    }
}
