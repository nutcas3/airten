use crate::Sample;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RegressionResult {
    // Test name
    pub name: String,
    // Whether the test passed
    pub passed: bool,
    // Maximum difference from expected
    pub max_diff: f32,
    // RMS difference from expected
    pub rms_diff: f32,
}

pub fn compare_against_golden(
    output: &[Sample],
    golden: &[Sample],
    tolerance: f32,
) -> RegressionResult {
    let name = "regression_test".to_string();
    
    if output.len() != golden.len() {
        return RegressionResult {
            name,
            passed: false,
            max_diff: f32::INFINITY,
            rms_diff: f32::INFINITY,
        };
    }

    let mut max_diff = 0.0f32;
    let mut sum_sq_diff = 0.0f32;

    for (out, gold) in output.iter().zip(golden.iter()) {
        let diff = (out - gold).abs();
        max_diff = max_diff.max(diff);
        sum_sq_diff += diff * diff;
    }

    let rms_diff = (sum_sq_diff / output.len() as f32).sqrt();
    let passed = max_diff < tolerance;

    RegressionResult {
        name,
        passed,
        max_diff,
        rms_diff,
    }
}

// Save audio as golden reference
pub fn save_golden_reference(samples: &[Sample], path: &Path) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;
    
    let mut file = File::create(path)?;
    
    // Write as binary f32
    for &sample in samples {
        file.write_all(&sample.to_le_bytes())?;
    }
    
    Ok(())
}

// Load golden reference
pub fn load_golden_reference(path: &Path) -> std::io::Result<Vec<Sample>> {
    use std::fs::File;
    use std::io::Read;
    
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let mut samples = Vec::with_capacity(buffer.len() / 4);
    for chunk in buffer.chunks_exact(4) {
        let bytes = [chunk[0], chunk[1], chunk[2], chunk[3]];
        samples.push(f32::from_le_bytes(bytes));
    }
    
    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_identical() {
        let samples = vec![1.0, 2.0, 3.0];
        let result = compare_against_golden(&samples, &samples, 0.001);
        
        assert!(result.passed);
        assert_eq!(result.max_diff, 0.0);
        assert_eq!(result.rms_diff, 0.0);
    }

    #[test]
    fn test_compare_different() {
        let output = vec![1.0, 2.0, 3.0];
        let golden = vec![1.1, 2.1, 3.1];
        let result = compare_against_golden(&output, &golden, 0.05);
        
        assert!(!result.passed);
        assert!((result.max_diff - 0.1).abs() < 0.001);
    }
}
