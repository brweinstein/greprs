use regex::Regex;
use std::fs;
use std::path::Path;

pub fn visit_path(regex: &Regex, path: &Path, print_filename: bool) {
    if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                visit_path(regex, &entry.path(), true);
            }
        } else {
            eprintln!("Failed to read directory: {}", path.display());
        }
    } else {
        search_file(regex, path, print_filename);
    }
}

fn highlight_matches(line: &str, regex: &Regex) -> String {
    let mut result = String::new();
    let mut last_index = 0;

    for mat in regex.find_iter(line) {
        result.push_str(&line[last_index..mat.start()]);
        result.push_str(&format!("\x1b[0;31m{}\x1b[0m", &line[mat.start()..mat.end()]));
        last_index = mat.end();
    }
    result.push_str(&line[last_index..]);
    result
}

fn search_file(regex: &Regex, path: &Path, print_filename: bool) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("greprs: {}: Could not read file", path.display());
            return;
        }
    };

    for line in content.lines() {
        if regex.is_match(line) {
            if print_filename {
                print!("{}:", path.display());
            }
            println!("{}", highlight_matches(line, regex));
        }
    }
}
