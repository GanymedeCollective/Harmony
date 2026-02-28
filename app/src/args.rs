use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "IRC-Discord bridge")]
pub struct Args {
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[arg(long)]
    pub log_path: Option<PathBuf>,

    #[arg(short, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// fetch channel/user to write fetched_data.toml
    Fetch,
}
