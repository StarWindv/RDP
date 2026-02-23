//! Build script to compile both versions

fn main() {
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/main_ssa.rs");
}