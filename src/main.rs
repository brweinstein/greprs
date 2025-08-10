mod cli;
mod search;
mod utils;

use clap::Parser;
use cli::Cli;
use rayon::prelude::*;
use search::{SearchConfig, visit_path};
use utils::{build_regex, RegexConfig};
use std::io::{self, Write};
use std::sync::Arc;

fn main() {
    let cli = Cli::parse();

    let regex_config = RegexConfig {
        ignore_case: cli.ignore_case,
        word_regexp: cli.word_regexp,
        line_regexp: cli.line_regexp,
        fixed_strings: cli.fixed_strings,
        extended_regexp: cli.extended_regexp,
    };

    let regex = match build_regex(&cli.pattern, &regex_config) {
        Ok(r) => r,
        Err(err) => {
            eprintln!("Invalid regex pattern: {}", err);
            std::process::exit(1);
        }
    };

    let search_config = SearchConfig {
        invert_match: cli.invert_match,
        line_number: cli.line_number,
        with_filename: cli.with_filename || cli.paths.len() > 1,
        no_filename: cli.no_filename,
        count: cli.count,
        files_with_matches: cli.files_with_matches,
        files_without_match: cli.files_without_match,
    };

    // Create a thread-safe stdout wrapper for parallel processing
    let stdout = Arc::new(std::sync::Mutex::new(io::stdout()));

    // Process paths in parallel with optimized buffering
    cli.paths.par_iter().for_each(|path| {
        let mut buffer = Vec::new();
        if let Err(err) = visit_path(&regex, path, &search_config, cli.recursive, &mut buffer) {
            eprintln!("greprs: {}: {}", path.display(), err);
        } else if !buffer.is_empty() {
            // Only lock stdout when we have output to write
            let mut stdout = stdout.lock().unwrap();
            let _ = stdout.write_all(&buffer);
        }
    });
}
