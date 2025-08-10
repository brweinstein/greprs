use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use glob::Pattern as GlobPattern;
use memmap2::Mmap;

#[derive(Debug, Default)]
pub struct SearchConfig {
    pub invert_match: bool,
    pub line_number: bool,
    pub with_filename: bool,
    pub no_filename: bool,
    pub count: bool,
    pub files_with_matches: bool,
    pub files_without_match: bool,
    pub only_matching: bool,
    pub quiet: bool,
    pub max_count: Option<usize>,
    pub after_context: Option<usize>,
    pub before_context: Option<usize>,
    pub context: Option<usize>,
    pub byte_offset: bool,
    pub null_data: bool,
    pub null: bool,
    pub text: bool,
    pub ignore_binary: bool,
    pub no_messages: bool,
    pub exclude_patterns: Vec<GlobPattern>,
    pub include_patterns: Vec<GlobPattern>,
    pub use_color: bool,
}

impl SearchConfig {
    pub fn effective_before_context(&self) -> usize {
        self.context.or(self.before_context).unwrap_or(0)
    }
    
    pub fn effective_after_context(&self) -> usize {
        self.context.or(self.after_context).unwrap_or(0)
    }
    
    pub fn has_context(&self) -> bool {
        self.effective_before_context() > 0 || self.effective_after_context() > 0
    }
}

pub fn visit_path<W: Write>(
    regex: &Regex,
    path: &Path,
    config: &SearchConfig,
    recursive: bool,
    writer: &mut W
) -> io::Result<()> {
    if path.is_dir() {
        let entries: Result<Vec<_>, io::Error> = fs::read_dir(path)?
            .collect();
        let entries = entries?;
        
        if recursive {
            // Use parallel processing only for larger directory sets
            if entries.len() > 20 {
                let results: Vec<Vec<u8>> = entries
                    .into_par_iter()
                    .filter_map(|entry| {
                        let path = entry.path();
                        if should_process_file(&path, config) || path.is_dir() {
                            let mut buffer = Vec::with_capacity(4096);
                            if let Err(err) = visit_path(regex, &path, config, recursive, &mut buffer) {
                                if !config.no_messages {
                                    eprintln!("greprs: {}: {}", path.display(), err);
                                }
                            }
                            if !buffer.is_empty() {
                                Some(buffer)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                
                for result in results {
                    writer.write_all(&result)?;
                }
            } else {
                // Sequential processing for smaller sets to avoid overhead
                for entry in entries {
                    let path = entry.path();
                    if should_process_file(&path, config) || path.is_dir() {
                        visit_path(regex, &path, config, recursive, writer)?;
                    }
                }
            }
        } else {
            // Non-recursive: only search files in the current directory
            for entry in entries {
                let path = entry.path();
                if path.is_file() && should_process_file(&path, config) {
                    search_file(regex, &path, config, writer)?;
                }
            }
        }
    } else {
        if should_process_file(path, config) {
            search_file(regex, path, config, writer)?;
        }
    }
    Ok(())
}

fn should_process_file(path: &Path, config: &SearchConfig) -> bool {
    // Early exit for non-files
    if !path.is_file() {
        return false;
    }
    
    let filename = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };
    
    // Check exclude patterns first (more common case)
    for pattern in &config.exclude_patterns {
        if pattern.matches(filename) {
            return false;
        }
    }
    
    // Check include patterns (if any specified, file must match at least one)
    if !config.include_patterns.is_empty() {
        let matches_include = config.include_patterns.iter()
            .any(|pattern| pattern.matches(filename));
        if !matches_include {
            return false;
        }
    }
    
    // Quick binary detection without reading file if ignore_binary is set
    if config.ignore_binary && !config.text {
        // Simple heuristic: check file extension first
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext {
                "bin" | "exe" | "dll" | "so" | "dylib" | "o" | "a" | "lib" | 
                "jpg" | "jpeg" | "png" | "gif" | "pdf" | "zip" | "tar" | "gz" => {
                    return false;
                }
                _ => {}
            }
        }
    }
    
    true
}

pub fn search_file<W: Write>(
    regex: &Regex,
    path: &Path,
    config: &SearchConfig,
    writer: &mut W
) -> io::Result<()> {
    let metadata = fs::metadata(path)?;
    let file_size = metadata.len() as usize;
    
    if file_size == 0 {
        return Ok(());
    }
    
    // Use memory mapping for larger files to avoid loading into memory
    if file_size > 1024 * 1024 {
        return search_mmap_file(regex, path, config, writer);
    }
    
    // For smaller files, read into string with pre-allocated capacity
    let mut file = fs::File::open(path)?;
    let mut contents = String::with_capacity(file_size);
    file.read_to_string(&mut contents)?;
    
    if config.null_data {
        let lines: Vec<&str> = contents.split('\0').collect();
        search_lines_optimized(regex, path, &lines, config, writer)
    } else {
        let lines: Vec<&str> = contents.lines().collect();
        search_lines_optimized(regex, path, &lines, config, writer)
    }
}

fn search_mmap_file<W: Write>(
    regex: &Regex,
    path: &Path,
    config: &SearchConfig,
    writer: &mut W
) -> io::Result<()> {
    let file = fs::File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let contents = std::str::from_utf8(&mmap).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidData, "File contains invalid UTF-8")
    })?;
    
    if config.null_data {
        let lines: Vec<&str> = contents.split('\0').collect();
        search_lines_optimized(regex, path, &lines, config, writer)
    } else {
        let lines: Vec<&str> = contents.lines().collect();
        search_lines_optimized(regex, path, &lines, config, writer)
    }
}

fn search_lines_optimized<W: Write>(
    regex: &Regex,
    path: &Path,
    lines: &[&str],
    config: &SearchConfig,
    writer: &mut W
) -> io::Result<()> {
    let mut count = 0;
    
    // Pre-compute these to avoid repeated checks
    let show_filename = config.with_filename && !config.no_filename;
    let has_context = config.has_context();
    
    // Early exit optimizations for simple cases
    if config.quiet {
        // For quiet mode, just check if any line matches
        for line in lines {
            if regex.is_match(line) != config.invert_match {
                return Ok(());
            }
        }
        return Ok(());
    }
    
    if config.files_with_matches {
        // For -l flag, just check if any line matches
        for line in lines {
            if regex.is_match(line) != config.invert_match {
                writeln!(writer, "{}", path.display())?;
                return Ok(());
            }
        }
        return Ok(());
    }
    
    if config.files_without_match {
        // For -L flag, check if no lines match
        for line in lines {
            if regex.is_match(line) != config.invert_match {
                return Ok(()); // Found a match, don't print filename
            }
        }
        writeln!(writer, "{}", path.display())?;
        return Ok(());
    }
    
    if config.count {
        // For count mode, just count matches
        for line in lines {
            if regex.is_match(line) != config.invert_match {
                count += 1;
                if let Some(max) = config.max_count {
                    if count >= max {
                        break;
                    }
                }
            }
        }
        if show_filename {
            write!(writer, "{}:", path.display())?;
        }
        writeln!(writer, "{}", count)?;
        return Ok(());
    }
    
    // Full search with context handling
    if has_context {
        search_with_context(regex, path, lines, config, writer)
    } else {
        search_without_context(regex, path, lines, config, writer)
    }
}

fn search_without_context<W: Write>(
    regex: &Regex,
    path: &Path,
    lines: &[&str],
    config: &SearchConfig,
    writer: &mut W
) -> io::Result<()> {
    let mut count = 0;
    let show_filename = config.with_filename && !config.no_filename;
    
    let mut byte_pos = 0;
    for (line_num, line) in lines.iter().enumerate() {
        let is_match = regex.is_match(line) != config.invert_match;
        
        if is_match {
            count += 1;
            
            if let Some(max) = config.max_count {
                if count > max {
                    break;
                }
            }
            
            if config.only_matching {
                print_only_matches_fast(writer, path, line_num, line, regex, config, byte_pos)?;
            } else {
                print_line_fast(writer, path, line_num, line, config, show_filename, byte_pos)?;
            }
        }
        
        if config.byte_offset {
            byte_pos += line.len() + 1; // +1 for newline
        }
    }
    
    Ok(())
}

fn search_with_context<W: Write>(
    regex: &Regex,
    path: &Path,
    lines: &[&str],
    config: &SearchConfig,
    writer: &mut W
) -> io::Result<()> {
    let mut count = 0;
    let show_filename = config.with_filename && !config.no_filename;
    let before_context = config.effective_before_context();
    let after_context = config.effective_after_context();
    
    let mut context_buffer: std::collections::VecDeque<(usize, &str)> = 
        std::collections::VecDeque::with_capacity(before_context);
    let mut after_lines_remaining = 0;
    let mut last_match_line = None;
    
    for (line_num, line) in lines.iter().enumerate() {
        let is_match = regex.is_match(line) != config.invert_match;
        
        if is_match {
            count += 1;
            
            if let Some(max) = config.max_count {
                if count > max {
                    break;
                }
            }
            
            // Print before context
            if before_context > 0 {
                for (ctx_line_num, ctx_line) in &context_buffer {
                    if Some(*ctx_line_num) != last_match_line {
                        print_line_fast(writer, path, *ctx_line_num, ctx_line, config, show_filename, 0)?;
                    }
                }
            }
            
            // Print the matching line
            if config.only_matching {
                print_only_matches_fast(writer, path, line_num, line, regex, config, 0)?;
            } else {
                print_line_fast(writer, path, line_num, line, config, show_filename, 0)?;
            }
            
            last_match_line = Some(line_num);
            after_lines_remaining = after_context;
        } else if after_lines_remaining > 0 {
            // Print after context
            print_line_fast(writer, path, line_num, line, config, show_filename, 0)?;
            after_lines_remaining -= 1;
        }
        
        // Maintain before context buffer
        if before_context > 0 {
            context_buffer.push_back((line_num, line));
            if context_buffer.len() > before_context {
                context_buffer.pop_front();
            }
        }
    }
    
    Ok(())
}

// Optimized print functions that avoid redundant checks
fn print_line_fast<W: Write>(
    writer: &mut W,
    path: &Path,
    line_num: usize,
    line: &str,
    config: &SearchConfig,
    show_filename: bool,
    byte_offset: usize,
) -> io::Result<()> {
    // Build output in a single write to reduce syscalls
    let mut output = Vec::with_capacity(line.len() + 64);
    
    if show_filename {
        if config.use_color {
            output.extend_from_slice(b"\x1b[35m");
            write!(&mut output, "{}", path.display())?;
            output.extend_from_slice(b"\x1b[0m:");
        } else {
            write!(&mut output, "{}:", path.display())?;
        }
    }
    
    if config.line_number {
        if config.use_color {
            output.extend_from_slice(b"\x1b[32m");
            write!(&mut output, "{}", line_num + 1)?;
            output.extend_from_slice(b"\x1b[0m:");
        } else {
            write!(&mut output, "{}:", line_num + 1)?;
        }
    }
    
    if config.byte_offset && byte_offset > 0 {
        write!(&mut output, "{}:", byte_offset)?;
    }
    
    if config.use_color {
        output.extend_from_slice(b"\x1b[1;31m");
        output.extend_from_slice(line.as_bytes());
        output.extend_from_slice(b"\x1b[0m");
    } else {
        output.extend_from_slice(line.as_bytes());
    }
    
    if config.null {
        output.push(0);
    } else {
        output.push(b'\n');
    }
    
    writer.write_all(&output)
}

fn print_only_matches_fast<W: Write>(
    writer: &mut W,
    path: &Path,
    line_num: usize,
    line: &str,
    regex: &Regex,
    config: &SearchConfig,
    byte_offset: usize,
) -> io::Result<()> {
    let show_filename = config.with_filename && !config.no_filename;
    
    for mat in regex.find_iter(line) {
        let mut output = Vec::with_capacity(mat.as_str().len() + 32);
        
        if show_filename {
            write!(&mut output, "{}:", path.display())?;
        }
        
        if config.line_number {
            write!(&mut output, "{}:", line_num + 1)?;
        }
        
        if config.byte_offset {
            write!(&mut output, "{}:", byte_offset + mat.start())?;
        }
        
        if config.use_color {
            output.extend_from_slice(b"\x1b[1;31m");
            output.extend_from_slice(mat.as_str().as_bytes());
            output.extend_from_slice(b"\x1b[0m");
        } else {
            output.extend_from_slice(mat.as_str().as_bytes());
        }
        
        if config.null {
            output.push(0);
        } else {
            output.push(b'\n');
        }
        
        writer.write_all(&output)?;
    }
    
    Ok(())
}


