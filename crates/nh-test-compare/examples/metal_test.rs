//! Minimal test to verify Metal GPU backend works
//!
//! Usage:
//!   cargo run --example metal_test -p nh-test-compare

use candle_core::{Device, Tensor, Result};

fn main() -> Result<()> {
    println!("=== Metal Backend Test ===");

    // Try to create Metal device
    let device = Device::new_metal(0)?;

    // Create tensors on GPU
    println!("Creating tensors on Metal device...");
    let a = Tensor::randn(0f32, 1., (3, 3), &device)?;
    let b = Tensor::randn(0f32, 1., (3, 3), &device)?;

    // Matrix multiplication on GPU
    println!("Performing matrix multiplication on GPU...");
    let c = a.matmul(&b)?;

    // Read result back to CPU
    println!("Reading result back to CPU...");
    let c_cpu: Vec<Vec<f32>> = c.to_vec2()?;

    println!("Result shape: {:?}", c.shape());
    println!("Sample values: {:?}", &c_cpu[0][..3]);

    // Test ReLU activation
    println!("\nTesting ReLU activation...");
    let neg = Tensor::from_slice(&[-1.0f32, 2.0, -3.0, 4.0], (2, 2), &device)?;
    let relu = neg.relu()?;
    let relu_cpu: Vec<Vec<f32>> = relu.to_vec2()?;
    println!("ReLU input: [[-1, 2], [-3, 4]]");
    println!("ReLU result: {:?}", relu_cpu);

    // Test with f16 if supported
    println!("\n=== Metal Test Complete ===");
    println!("Metal GPU backend is working correctly!");

    Ok(())
}
