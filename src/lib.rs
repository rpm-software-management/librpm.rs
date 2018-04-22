//! Rust binding for librpm: the RPM Package Manager library

#![crate_name = "librpm"]
#![crate_type = "rlib"]
#![deny(warnings, missing_docs, trivial_casts, trivial_numeric_casts)]
#![deny(unused_import_braces, unused_qualifications)]
#![doc(html_root_url = "https://librpm.rs/librpm/")]

extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate librpm_sys;

#[macro_use]
pub mod error;
