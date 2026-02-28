use std::fs::OpenOptions;
use std::path::Path;

fn verbosity_to_level(v: u8) -> log::LevelFilter {
    match v {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    }
}

pub fn init(verbosity: u8, log_path: Option<&Path>) {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(verbosity_to_level(verbosity));

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
