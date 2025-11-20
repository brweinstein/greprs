use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "greprs")]
#[command(about = "A fast grep clone written in Rust")]
#[command(version)]
#[command(disable_help_flag = true)]
pub struct CliArgs {
    /// Print help information
    #[arg(long = "help", action = clap::ArgAction::Help)]
    pub help: Option<bool>,
    /// Pattern to search for
    pub pattern: String,

    /// Files or directories to search
    pub files: Vec<PathBuf>,

    /// Ignore case distinctions in patterns and input data
    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,

    /// Treat PATTERNS as fixed strings, not regular expressions
    #[arg(short = 'F', long = "fixed-strings")]
    pub fixed_strings: bool,

    /// Select only lines containing matches that form whole words
    #[arg(short = 'w', long = "word-regexp")]
    pub word_regexp: bool,

    /// Select only lines containing matches that match the entire line
    #[arg(short = 'x', long = "line-regexp")]
    pub line_regexp: bool,

    /// Select non-matching lines
    #[arg(short = 'v', long = "invert-match")]
    pub invert_match: bool,

    /// Print line number with output lines
    #[arg(short = 'n', long = "line-number")]
    pub line_number: bool,

    /// Print only a count of selected lines per file
    #[arg(short = 'c', long = "count")]
    pub count: bool,

    /// Print only names of files with selected lines
    #[arg(short = 'l', long = "files-with-matches")]
    pub files_with_matches: bool,

    /// Print only names of files with no selected lines
    #[arg(short = 'L', long = "files-without-match")]
    pub files_without_match: bool,

    /// Suppress the prefixing of file names on output
    #[arg(short = 'h', long = "no-filename")]
    pub no_filename: bool,

    /// Always print file names for matches
    #[arg(short = 'H', long = "with-filename")]
    pub with_filename: bool,

    /// Read directories recursively
    #[arg(short = 'r', long = "recursive")]
    pub recursive: bool,

    /// Print only the matched parts of matching lines
    #[arg(short = 'o', long = "only-matching")]
    pub only_matching: bool,

    /// Suppress normal output; exit with 0 if any match found
    #[arg(short = 'q', long = "quiet", alias = "silent")]
    pub quiet: bool,

    /// Stop after NUM matches
    #[arg(short = 'm', long = "max-count", value_name = "NUM")]
    pub max_count: Option<usize>,

    /// Show NUM lines after each match
    #[arg(short = 'A', long = "after-context", value_name = "NUM")]
    pub after_context: Option<usize>,

    /// Show NUM lines before each match  
    #[arg(short = 'B', long = "before-context", value_name = "NUM")]
    pub before_context: Option<usize>,

    /// Show NUM lines before and after each match
    #[arg(short = 'C', long = "context", value_name = "NUM")]
    pub context: Option<usize>,

    /// Use colored output (auto/always/never)
    #[arg(long = "color", value_enum, default_value = "never")]
    pub color: ColorOption,

    /// Skip files whose base name matches GLOB
    #[arg(long = "exclude", value_name = "GLOB")]
    pub exclude: Vec<String>,

    /// Search only files whose base name matches GLOB
    #[arg(long = "include", value_name = "GLOB")]
    pub include: Vec<String>,

    /// Follow symbolic links
    #[arg(short = 'R', long = "dereference-recursive")]
    pub dereference_recursive: bool,

    /// Process binary files as if they were text
    #[arg(short = 'a', long = "text")]
    pub text: bool,

    /// Skip binary files
    #[arg(short = 'I', long = "ignore-binary")]
    pub ignore_binary: bool,

    /// Print byte offset of each match
    #[arg(short = 'b', long = "byte-offset")]
    pub byte_offset: bool,

    /// Read patterns from FILE
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    pub pattern_file: Option<PathBuf>,

    /// Suppress error messages about nonexistent or unreadable files
    #[arg(short = 's', long = "no-messages")]
    pub no_messages: bool,

    /// Use null character as record separator
    #[arg(long = "null-data")]
    pub null_data: bool,

    /// Use null character as line separator
    #[arg(short = 'z', long = "null")]
    pub null: bool,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ColorOption {
    Auto,
    Always,
    Never,
}
