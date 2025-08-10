use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "greprs", about = "A grep clone in Rust")]
pub struct Cli {
    /// Patterns to search for
    #[arg(name = "PATTERNS")]
    pub pattern: String,
    
    /// Files to search in
    #[arg(name = "FILE")]
    pub paths: Vec<PathBuf>,
    
    #[arg(short = 'E', long)]
    /// Use extended regular expressions
    pub extended_regexp: bool,
    
    #[arg(short = 'F', long)]
    /// Use fixed strings instead of regular expressions
    pub fixed_strings: bool,
    
    #[arg(short, long)]
    /// Ignore case distinctions
    pub ignore_case: bool,
    
    #[arg(short = 'v', long)]
    /// Select non-matching lines
    pub invert_match: bool,
    
    #[arg(short = 'w', long)]
    /// Match only whole words
    pub word_regexp: bool,
    
    #[arg(short = 'x', long)]
    /// Match only whole lines
    pub line_regexp: bool,
    
    #[arg(short = 'c', long)]
    /// Print only a count of matching lines
    pub count: bool,
    
    #[arg(short = 'n', long)]
    /// Print line numbers
    pub line_number: bool,
    
    #[arg(short = 'H', long)]
    /// Print filename with output lines
    pub with_filename: bool,
    
    #[arg(short = 'h', long)]
    /// Suppress filename prefix
    pub no_filename: bool,
    
    #[arg(short = 'r', long)]
    /// Recursively search directories
    pub recursive: bool,
    
    #[arg(short = 'l', long)]
    /// Print only names of files with matches
    pub files_with_matches: bool,
    
    #[arg(short = 'L', long)]
    /// Print only names of files without matches
    pub files_without_match: bool,
}
