//! bindgen configuration for librpmbuild-sys

use bindgen::Builder;
use std::{env, path::PathBuf};

/// Bind to librpmbuild.so
fn main() {
    println!("cargo:rustc-link-lib=rpmbuild");

    // TODO: whitelist types and functions we actually use
    let builder = Builder::default()
        .header("include/librpmbuild.hpp")
        .blacklist_type("timex");

    // Write generated bindings to OUT_DIR (to be included in the crate)
    let output_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("binding.rs");

    builder
        .generate()
        .unwrap()
        .write_to_file(output_path)
        .unwrap();
}
