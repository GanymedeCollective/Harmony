//! Logger

use std::fs::OpenOptions;
use std::path::Path;

const fn verbosity_to_level(v: u8) -> log::LevelFilter {
    match v {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    }
}

/// Init logger with the given `verbosity` and at the given `log_path` (if
/// specified).
///
/// # Panics
///
/// Panics if given `log_path` could not be used to create a log file.
pub fn init(verbosity: u8, log_path: Option<&Path>) {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(verbosity_to_level(verbosity));
    builder.filter_module("tracing::span", log::LevelFilter::Warn);

    if let Some(path) = log_path {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap_or_else(|e| panic!("failed to open log file {}: {e}", path.display()));
        builder.target(env_logger::Target::Pipe(Box::new(file)));
    } else {
        builder.target(env_logger::Target::Stderr);
    }

    builder.init();
}
