use clap::Parser;
use cli::{CliArgs, ColorOption};
use utils::{build_regex, RegexConfig};
use search::{SearchConfig, visit_path};
use std::io::{self, Write, BufWriter};

mod cli;
mod search;
mod utils;

fn main() -> io::Result<()> {
    let args = CliArgs::parse();
    
    // Handle context options
    let (before_context, after_context) = match args.context {
        Some(n) => (Some(n), Some(n)),
        None => (args.before_context, args.after_context),
    };
    
    // Determine color usage
    let use_color = match args.color {
        ColorOption::Always => true,
        ColorOption::Never => false,
        ColorOption::Auto => atty::is(atty::Stream::Stdout),
    };
    
    // Parse exclude/include patterns with better error handling
    let exclude_patterns: Vec<_> = args.exclude.iter()
        .filter_map(|s| glob::Pattern::new(s).ok())
        .collect();
    
    let include_patterns: Vec<_> = args.include.iter()
        .filter_map(|s| glob::Pattern::new(s).ok())
        .collect();
    
    let regex_config = RegexConfig {
        ignore_case: args.ignore_case,
        word_regexp: args.word_regexp,
        line_regexp: args.line_regexp,
        fixed_strings: args.fixed_strings,
    };
    
    let regex = build_regex(&args.pattern, &regex_config)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    
    // Auto-detect if we should show filenames (like grep does)
    // Show filenames if: multiple files OR --with-filename OR (not --no-filename AND multiple files)
    let should_show_filename = if args.no_filename {
        false
    } else if args.with_filename {
        true
    } else {
        args.files.len() > 1
    };
    
    let config = SearchConfig {
        invert_match: args.invert_match,
        line_number: args.line_number,
        with_filename: should_show_filename,
        count: args.count,
        files_with_matches: args.files_with_matches,
        files_without_match: args.files_without_match,
        only_matching: args.only_matching,
        quiet: args.quiet,
        max_count: args.max_count,
        before_context,
        after_context,
        context: args.context,
        byte_offset: args.byte_offset,
        null_data: args.null_data,
        null: args.null,
        text: args.text,
        ignore_binary: args.ignore_binary,
        no_messages: args.no_messages,
        exclude_patterns,
        include_patterns,
        use_color,
    };

    // Use buffered writer for better performance
    let stdout = io::stdout();
    let mut handle = BufWriter::with_capacity(64 * 1024, stdout.lock());

    if args.files.is_empty() {
        eprintln!("Reading from stdin not yet implemented, please provide file arguments");
        std::process::exit(1);
    } else {
        for file_path in &args.files {
            visit_path(&regex, file_path, &config, args.recursive, &mut handle)?;
        }
    }
    
    // Ensure all output is flushed
    handle.flush()?;

    Ok(())
}
