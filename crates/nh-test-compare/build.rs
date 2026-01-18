//! Build script for nh-test-compare
//!
//! Compiles the NetHack C FFI implementation for comparison testing.

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let nethack_src = std::path::PathBuf::from(&manifest_dir).join("nethack_src");
    let c_src = std::path::PathBuf::from(&manifest_dir).join("c_src");
    
    // Check if real NetHack source is available
    let real_nethack_src = std::path::PathBuf::from("/Users/pierre/src/games/NetHack-3.6.7/src");
    
    if real_nethack_src.exists() {
        println!("Building with real NetHack 3.6.7 source");
        
        // Build with real NetHack source - compile ALL C files in a single builder
        // to ensure all symbols are available for linking
        let mut builder = cc::Build::new();
        builder.opt_level(2);
        
        // Always need isaac64_standalone.c for FFI RNG tests
        builder.file(c_src.join("isaac64_standalone.c"));
        
        // Compile the FFI wrapper
        builder.file(nethack_src.join("nethack_ffi.c"));
        
        builder.compile("nethack_c");
    } else {
        println!("Building standalone NetHack FFI stub");
        
        // Build ISAAC64 RNG first
        let mut isaac_builder = cc::Build::new();
        isaac_builder.opt_level(2);
        isaac_builder.file(c_src.join("isaac64_standalone.c"));
        isaac_builder.compile("isaac64");
        
        // Build FFI library
        let mut ffi_builder = cc::Build::new();
        ffi_builder.opt_level(2);
        ffi_builder.file(nethack_src.join("nethack_ffi.c"));
        ffi_builder.compile("nethack_ffi");
    }
    
    // Print link information
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=nethack_src/");
    println!("cargo:rerun-if-changed=c_src/");
}
