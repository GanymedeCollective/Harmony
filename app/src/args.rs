use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(about = "IRC-Discord bridge")]
pub struct Args {
    #[arg(long)]
    pub log_path: Option<PathBuf>,

    #[arg(short, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

pub fn parse() -> Args {
    Args::parse()
}
