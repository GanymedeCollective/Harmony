//! Logger

use std::fs::OpenOptions;
use std::path::Path;

use tracing_subscriber::{EnvFilter, filter::LevelFilter, fmt::writer::BoxMakeWriter};

const fn verbosity_to_level(v: u8) -> LevelFilter {
    match v {
        0 => LevelFilter::INFO,
        1 => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    }
}

/// Init logger with the given `verbosity` and at the given `log_path` (if
/// specified).
///
/// # Panics
///
/// Panics if given `log_path` could not be used to create a log file.
pub fn init(verbosity: u8, log_path: Option<&Path>) {
    let verbosity = verbosity_to_level(verbosity);
    let filter = EnvFilter::builder()
        .with_default_directive(verbosity.into())
        .from_env()
        .unwrap_or_else(|_| EnvFilter::from(verbosity.to_string()));

    let writer = log_path.map_or_else(
        || BoxMakeWriter::new(std::io::stderr),
        |path| {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap_or_else(|e| panic!("failed to open log file {}: {e}", path.display()));
            BoxMakeWriter::new(file)
        },
    );

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_target(false)
        .init();
}
