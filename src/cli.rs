use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "greprs",
    about = "A barebones grep clone in Rust",
    author = "Ben Weinstein",
    version = "0.2.0"
)]
pub struct Cli {
    pub pattern: String,
    pub paths: Vec<PathBuf>,
    #[arg(short, long, default_value_t = false)]
    pub ignore_case: bool,
}
