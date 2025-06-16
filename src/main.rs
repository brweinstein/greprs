mod cli;
mod search;
mod utils;

use clap::Parser;
use cli::Cli;
use rayon::prelude::*;
use search::visit_path;
use utils::build_regex;

fn main() {
    let cli = Cli::parse();

    let regex = match build_regex(&cli.pattern, cli.ignore_case) {
        Ok(r) => r,
        Err(err) => {
            eprintln!("Invalid regex pattern: {}", err);
            std::process::exit(1);
        }
    };

    let print_filename = cli.paths.len() > 1 || cli.paths.iter().any(|p| p.is_dir());

    cli.paths.par_iter().for_each(|path| {
        visit_path(&regex, path, print_filename);
    });
}
