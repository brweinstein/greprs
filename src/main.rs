// Basic Grep Utility
// Used to learn ANSI, basic CLI usage, error handling, recursive directory travel
// Ben Weinstein

// Imports
use std::env;
use std::path::PathBuf;
use std::process;

// Cli Struct for Arguments
struct Cli {
    pattern: String,
    paths: Vec<PathBuf>,
}

// Error handling for invalid arguments
fn usage_and_exit() {
    eprintln!("Invalid arguments provided.");
    eprintln!("Usage: greprs [PATTERN] [PATH1] [PATH2] ...");
    process::exit(1);
}

// Search file for pattern
// File is never a directory
fn search_file(pattern: String, path: PathBuf) {
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("greprs: {:?}: No such file or directory", path);
            process::exit(1);
        }
    };
    //ANSI Escape Codes for formatting
    let red_pattern = format!("\x1b[0;31m{}\x1b[0m", &pattern);
    if content.contains(&pattern) {
        print!("\x1b[0;35m{:?}\x1b[0;36m:\x1b[0m", path);
        println!("{}", content.replace(&pattern.clone(), &red_pattern));
    }
}

//Handle recursive search and check path vs directory
fn visit_path(pattern: String, path: PathBuf) {
    if path.is_dir() {
        let entries = match std::fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(msg) => {
                eprintln!("Failed to read directory {:?}: {}", path, msg);
                return;
            }
        };
        // As file tree is non-linear, use flatten to iterate
        for entry in entries.flatten() {
            visit_path(pattern.clone(), entry.path());
        }
    } else { //File
        search_file(pattern, path);
    } 
}

fn main() {
    // greprs pattern ./path/to/file
    let mut args = env::args::skip(1);

    let pattern = match args.next() {
        Some(pattern) => pattern,
        None => {
            usage_and_exit();
            return;
        }
    };

    let paths: Vec<PathBuf> = args.map(PathBuf::from).collect();
    if paths.is_empty() {
        usage_and_exit();
    }
    // Create an instance of Cli struct
    let cli = Cli { pattern, paths };

    for path in cli.paths {
        visit_path(cli.pattern.clone(), path)
    }
}
