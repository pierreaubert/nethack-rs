//! Build script for nh-test-compare
//!
//! Compiles a standalone ISAAC64 implementation for FFI comparison testing.

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let c_src = std::path::PathBuf::from(manifest_dir).join("c_src");

    // Compile standalone ISAAC64 implementation
    cc::Build::new()
        .file(c_src.join("isaac64_standalone.c"))
        .opt_level(2)
        .compile("isaac64_c");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=c_src/isaac64_standalone.c");
}
