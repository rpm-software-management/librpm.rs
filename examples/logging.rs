//! Demonstrate librpm log messages routed through Rust's `log` crate.
//!
//! Run with: RUST_LOG=librpm=debug cargo run --example logging

use librpm::logging::{self, LogBehavior, LogLevel};

fn main() {
    env_logger::init();

    librpm::init().expect("failed to initialize librpm");

    // Route librpm messages through Rust's log crate instead of stderr
    logging::set_behavior(LogBehavior::LogCrate);
    log::info!("librpm logging routed through log crate");

    // Restrict librpm to only emit warnings and above
    logging::set_verbosity(LogLevel::Warning);

    // Retrieve the last log message from librpm (if any)
    if let Some(msg) = logging::last_message() {
        log::info!("last librpm message: {msg}");
    }
}
