use crate::Sample;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationType {
    Linear,
    ReLU,
    LeakyReLU,
    Sigmoid,
    Tanh,
    ELU,
    Softmax,
    Swish,
    GELU,
}


pub struct Activation {
    activation_type: ActivationType,
    alpha: Sample,
}

impl Activation {
    pub fn new(activation_type: ActivationType) -> Self {
        Self {
            activation_type,
            alpha: 0.01,
        }
    }

    pub fn leaky_relu(alpha: Sample) -> Self {
        Self {
            activation_type: ActivationType::LeakyReLU,
            alpha,
        }
    }

    pub fn elu(alpha: Sample) -> Self {
        Self {
            activation_type: ActivationType::ELU,
            alpha,
        }
    }

    #[inline]
    pub fn apply(&self, x: Sample) -> Sample {
        match self.activation_type {
            ActivationType::Linear => x,
            ActivationType::ReLU => x.max(0.0),
            ActivationType::LeakyReLU => {
                if x > 0.0 { x } else { self.alpha * x }
            }
            ActivationType::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            ActivationType::Tanh => x.tanh(),
            ActivationType::ELU => {
                if x > 0.0 { x } else { self.alpha * (x.exp() - 1.0) }
            }
            ActivationType::Softmax => x,
            ActivationType::Swish => x * (1.0 / (1.0 + (-x).exp())),
            ActivationType::GELU => {
                let sqrt_2_pi = 0.797_884_56;
                0.5 * x * (1.0 + (sqrt_2_pi * (x + 0.044715 * x * x * x)).tanh())
            }
        }
    }

    pub fn apply_inplace(&self, data: &mut [Sample]) {
        if self.activation_type == ActivationType::Softmax {
            self.apply_softmax(data);
        } else {
            for x in data.iter_mut() {
                *x = self.apply(*x);
            }
        }
    }

    fn apply_softmax(&self, data: &mut [Sample]) {
        let max = data.iter().cloned().fold(Sample::NEG_INFINITY, Sample::max);
        
        let mut sum = 0.0;
        for x in data.iter_mut() {
            *x = (*x - max).exp();
            sum += *x;
        }
        
        if sum > 0.0 {
            for x in data.iter_mut() {
                *x /= sum;
            }
        }
    }

    #[inline]
    pub fn derivative(&self, x: Sample) -> Sample {
        match self.activation_type {
            ActivationType::Linear => 1.0,
            ActivationType::ReLU => if x > 0.0 { 1.0 } else { 0.0 },
            ActivationType::LeakyReLU => if x > 0.0 { 1.0 } else { self.alpha },
            ActivationType::Sigmoid => {
                let s = self.apply(x);
                s * (1.0 - s)
            }
            ActivationType::Tanh => {
                let t = x.tanh();
                1.0 - t * t
            }
            ActivationType::ELU => {
                if x > 0.0 { 1.0 } else { self.apply(x) + self.alpha }
            }
            ActivationType::Softmax => 1.0,
            ActivationType::Swish => {
                let s = 1.0 / (1.0 + (-x).exp());
                s + x * s * (1.0 - s)
            }
            ActivationType::GELU => {
                let sqrt_2_pi = 0.797_884_56;
                let inner = sqrt_2_pi * (x + 0.044715 * x * x * x);
                let tanh_inner = inner.tanh();
                let sech2 = 1.0 - tanh_inner * tanh_inner;
                0.5 * (1.0 + tanh_inner) + 0.5 * x * sech2 * sqrt_2_pi * (1.0 + 0.134145 * x * x)
            }
        }
    }
}

impl Default for Activation {
    fn default() -> Self {
        Self::new(ActivationType::ReLU)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relu() {
        let act = Activation::new(ActivationType::ReLU);
        assert_eq!(act.apply(1.0), 1.0);
        assert_eq!(act.apply(-1.0), 0.0);
        assert_eq!(act.apply(0.0), 0.0);
    }

    #[test]
    fn test_leaky_relu() {
        let act = Activation::leaky_relu(0.1);
        assert_eq!(act.apply(1.0), 1.0);
        assert!((act.apply(-1.0) - (-0.1)).abs() < 0.001);
    }

    #[test]
    fn test_sigmoid() {
        let act = Activation::new(ActivationType::Sigmoid);
        assert!((act.apply(0.0) - 0.5).abs() < 0.001);
        assert!(act.apply(10.0) > 0.99);
        assert!(act.apply(-10.0) < 0.01);
    }

    #[test]
    fn test_tanh() {
        let act = Activation::new(ActivationType::Tanh);
        assert!((act.apply(0.0) - 0.0).abs() < 0.001);
        assert!(act.apply(10.0) > 0.99);
        assert!(act.apply(-10.0) < -0.99);
    }

    #[test]
    fn test_softmax() {
        let act = Activation::new(ActivationType::Softmax);
        let mut data = [1.0, 2.0, 3.0];
        act.apply_inplace(&mut data);
        
        let sum: Sample = data.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);
        
        assert!(data[2] > data[1]);
        assert!(data[1] > data[0]);
    }

    #[test]
    fn test_derivatives() {
        let relu = Activation::new(ActivationType::ReLU);
        assert_eq!(relu.derivative(1.0), 1.0);
        assert_eq!(relu.derivative(-1.0), 0.0);

        let sigmoid = Activation::new(ActivationType::Sigmoid);
        let d = sigmoid.derivative(0.0);
        assert!((d - 0.25).abs() < 0.001);
    }
}
