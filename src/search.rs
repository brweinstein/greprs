use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

#[derive(Debug, Default)]
pub struct SearchConfig {
    pub invert_match: bool,
    pub line_number: bool,
    pub with_filename: bool,
    pub no_filename: bool,
    pub count: bool,
    pub files_with_matches: bool,
    pub files_without_match: bool,
}

pub fn visit_path<W: Write>(
    regex: &Regex,
    path: &Path,
    config: &SearchConfig,
    recursive: bool,
    writer: &mut W
) -> io::Result<()> {
    if path.is_dir() {
        if recursive {
            // Collect all entries first
            let entries: Result<Vec<_>, io::Error> = fs::read_dir(path)?
                .collect();
            let entries = entries?;
            
            // Use parallel processing for directories with many files
            if entries.len() > 10 {
                let results: Vec<Vec<u8>> = entries
                    .into_par_iter()
                    .map(|entry| {
                        let mut buffer = Vec::new();
                        if let Err(err) = visit_path(regex, &entry.path(), config, recursive, &mut buffer) {
                            eprintln!("Error processing {}: {}", entry.path().display(), err);
                        }
                        buffer
                    })
                    .collect();
                
                // Write all results sequentially
                for result in results {
                    if !result.is_empty() {
                        writer.write_all(&result)?;
                    }
                }
            } else {
                // For small directories, process sequentially
                for entry in entries {
                    visit_path(regex, &entry.path(), config, recursive, writer)?;
                }
            }
        } else {
            writeln!(writer, "greprs: {}: Is a directory", path.display())?;
        }
    } else {
        search_file(regex, path, config, writer)?;
    }
    Ok(())
}

pub fn search_file<W: Write>(
    regex: &Regex,
    path: &Path,
    config: &SearchConfig,
    writer: &mut W
) -> io::Result<()> {
    // Try to get file size to optimize memory allocation
    let metadata = fs::metadata(path)?;
    let file_size = metadata.len() as usize;
    
    // Skip empty files
    if file_size == 0 {
        return Ok(());
    }
    
    // For very large files, use a different strategy
    if file_size > 50 * 1024 * 1024 { // 50MB
        return search_large_file(regex, path, config, writer);
    }
    
    let mut file = fs::File::open(path)?;
    let mut contents = String::with_capacity(file_size.min(1024 * 1024)); // Cap at 1MB for initial allocation
    file.read_to_string(&mut contents)?;
    
    let mut count = 0;
    let mut has_match = false;
    let show_filename = config.with_filename && !config.no_filename;
    
    // Fast path: just count matches without processing lines
    if config.count || config.files_with_matches || config.files_without_match {
        for line in contents.lines() {
            if regex.is_match(line) != config.invert_match {
                count += 1;
                has_match = true;
                if config.files_with_matches {
                    break; // Early exit for -l flag
                }
            }
        }
    } else {
        // Full processing with output - use pre-allocated buffer for performance
        let mut output_buffer = Vec::with_capacity(1024);
        for (line_num, line) in contents.lines().enumerate() {
            if regex.is_match(line) != config.invert_match {
                count += 1;
                has_match = true;
                
                output_buffer.clear();
                if show_filename {
                    output_buffer.extend_from_slice(path.to_string_lossy().as_bytes());
                    output_buffer.push(b':');
                }
                if config.line_number {
                    output_buffer.extend_from_slice((line_num + 1).to_string().as_bytes());
                    output_buffer.push(b':');
                }
                output_buffer.extend_from_slice(line.as_bytes());
                output_buffer.push(b'\n');
                
                writer.write_all(&output_buffer)?;
            }
        }
    }

    // Output final results
    if config.count {
        if show_filename {
            write!(writer, "{}:", path.display())?;
        }
        writeln!(writer, "{}", count)?;
    } else if config.files_with_matches && has_match {
        writeln!(writer, "{}", path.display())?;
    } else if config.files_without_match && !has_match {
        writeln!(writer, "{}", path.display())?;
    }

    Ok(())
}

// Handle very large files with streaming to avoid memory issues
fn search_large_file<W: Write>(
    regex: &Regex,
    path: &Path,
    config: &SearchConfig,
    writer: &mut W
) -> io::Result<()> {
    use std::io::{BufRead, BufReader};
    
    let file = fs::File::open(path)?;
    let reader = BufReader::with_capacity(64 * 1024, file); // 64KB buffer
    let mut count = 0;
    let mut has_match = false;
    let show_filename = config.with_filename && !config.no_filename;

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        if regex.is_match(&line) != config.invert_match {
            count += 1;
            has_match = true;

            if !config.count && !config.files_with_matches && !config.files_without_match {
                if show_filename {
                    write!(writer, "{}:", path.display())?;
                }
                if config.line_number {
                    write!(writer, "{}:", line_num + 1)?;
                }
                writeln!(writer, "{}", line)?;
            } else if config.files_with_matches {
                break; // Early exit for -l flag
            }
        }
    }

    if config.count {
        if show_filename {
            write!(writer, "{}:", path.display())?;
        }
        writeln!(writer, "{}", count)?;
    } else if config.files_with_matches && has_match {
        writeln!(writer, "{}", path.display())?;
    } else if config.files_without_match && !has_match {
        writeln!(writer, "{}", path.display())?;
    }

    Ok(())
}


