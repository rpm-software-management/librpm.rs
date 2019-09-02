//! bindgen configuration for librpmsign-sys

use bindgen::Builder;
use std::{env, path::PathBuf};

/// Bind to librpmsign.so
fn main() {
    println!("cargo:rustc-link-lib=rpmsign");

    // Write generated bindings to OUT_DIR (to be included in the crate)
    let output_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("binding.rs");

    // TODO: whitelist types and functions we actually use
    Builder::default()
        .header("include/librpmsign.hpp")
        .generate()
        .unwrap()
        .write_to_file(output_path)
        .unwrap();
}
