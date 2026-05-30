use crate::Sample;
use crate::nn::activation::{Activation, ActivationType};

/// Neural network layer with weights, biases, and activation function
///
/// Uses static references to weights and biases for no-heap operation.
pub struct Layer {
    weights: &'static [Sample],
    biases: &'static [Sample],
    input_size: usize,
    output_size: usize,
    activation: Activation,
}

impl Layer {
    /// Creates a new layer with the given weights, biases, and sizes
    ///
    /// # Arguments
    /// * `weights` - Static reference to weight matrix
    /// * `biases` - Static reference to bias vector
    /// * `input_size` - Number of input neurons
    /// * `output_size` - Number of output neurons
    pub const fn new(
        weights: &'static [Sample],
        biases: &'static [Sample],
        input_size: usize,
        output_size: usize,
    ) -> Self {
        Self {
            weights,
            biases,
            input_size,
            output_size,
            activation: Activation {
                activation_type: ActivationType::ReLU,
                alpha: 0.01,
            },
        }
    }

    /// Sets the activation function for this layer
    pub fn with_activation(mut self, activation: Activation) -> Self {
        self.activation = activation;
        self
    }

    /// Returns the input size of this layer
    #[inline]
    pub fn input_size(&self) -> usize {
        self.input_size
    }

    /// Returns the output size of this layer
    #[inline]
    pub fn output_size(&self) -> usize {
        self.output_size
    }

    /// Performs forward pass through this layer with activation
    ///
    /// # Arguments
    /// * `input` - Input samples
    /// * `output` - Output buffer (must be at least `output_size` long)
    pub fn forward(&self, input: &[Sample], output: &mut [Sample]) {
        debug_assert_eq!(input.len(), self.input_size);
        debug_assert_eq!(output.len(), self.output_size);

        for i in 0..self.output_size {
            let mut sum = self.biases[i];
            let weight_offset = i * self.input_size;

            for j in 0..self.input_size {
                sum += input[j] * self.weights[weight_offset + j];
            }

            output[i] = self.activation.apply(sum);
        }
    }

    /// Performs linear forward pass (matrix multiplication + bias) without activation
    ///
    /// # Arguments
    /// * `input` - Input samples
    /// * `output` - Output buffer (must be at least `output_size` long)
    pub fn forward_linear(&self, input: &[Sample], output: &mut [Sample]) {
        debug_assert_eq!(input.len(), self.input_size);
        debug_assert_eq!(output.len(), self.output_size);

        for i in 0..self.output_size {
            let mut sum = self.biases[i];
            let weight_offset = i * self.input_size;

            for j in 0..self.input_size {
                sum += input[j] * self.weights[weight_offset + j];
            }

            output[i] = sum;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_WEIGHTS: [Sample; 6] = [1.0, 0.0, 0.0, 1.0, 0.5, 0.5];
    static TEST_BIASES: [Sample; 2] = [0.0, 0.1];

    #[test]
    fn test_layer_forward() {
        let layer = Layer::new(&TEST_WEIGHTS, &TEST_BIASES, 3, 2)
            .with_activation(Activation::new(ActivationType::Linear));

        let input = [1.0, 2.0, 3.0];
        let mut output = [0.0; 2];

        layer.forward(&input, &mut output);

        assert!((output[0] - 1.0).abs() < 0.001);
        assert!((output[1] - 3.6).abs() < 0.001);
    }

    #[test]
    fn test_layer_with_relu() {
        let layer = Layer::new(&TEST_WEIGHTS, &TEST_BIASES, 3, 2);

        let input = [-1.0, -2.0, -3.0];
        let mut output = [0.0; 2];

        layer.forward(&input, &mut output);

        assert!(output[0] >= 0.0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_dynamic_layer() {
        let mut layer = DynamicLayer::new(2, 2);
        layer.set_weights(&[1.0, 0.0, 0.0, 1.0]);
        layer.set_biases(&[0.0, 0.0]);
        layer.set_activation(Activation::new(ActivationType::Linear));

        let input = [1.0, 2.0];
        let mut output = [0.0; 2];

        layer.forward(&input, &mut output);

        assert!((output[0] - 1.0).abs() < 0.001);
        assert!((output[1] - 2.0).abs() < 0.001);
    }

    static CONV_WEIGHTS: [Sample; 3] = [1.0, 0.0, -1.0];

    #[test]
    fn test_conv1d() {
        let conv = Conv1D::new(&CONV_WEIGHTS, 0.0, 3, 1);

        let input = [1.0, 2.0, 3.0, 4.0, 5.0];
        let mut output = [0.0; 3];

        conv.forward(&input, &mut output);

        assert_eq!(output[0], 0.0);
    }
}
