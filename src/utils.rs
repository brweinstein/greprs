use regex::{Regex, RegexBuilder};

pub fn build_regex(pattern: &str, config: &RegexConfig) -> Result<Regex, regex::Error> {
    let mut pattern = pattern.to_string();
    
    if config.fixed_strings {
        pattern = regex::escape(&pattern);
    } else {
        // Handle extended regex mode
        if !config.extended_regexp {
            // In basic mode (not extended), escape special characters
            pattern = escape_basic_regex(&pattern);
        }
        
        if config.word_regexp {
            pattern = format!(r"\b{}\b", pattern);
        }
        
        if config.line_regexp {
            pattern = format!("^{}$", pattern);
        }
    }
    
    RegexBuilder::new(&pattern)
        .case_insensitive(config.ignore_case)
        .build()
}

fn escape_basic_regex(pattern: &str) -> String {
    let special_chars = ['+', '?', '|', '(', ')', '{', '}'];
    let mut result = String::with_capacity(pattern.len() * 2);
    
    for c in pattern.chars() {
        if special_chars.contains(&c) {
            result.push('\\');
        }
        result.push(c);
    }
    result
}

#[derive(Debug)]
pub struct RegexConfig {
    pub ignore_case: bool,
    pub word_regexp: bool,
    pub line_regexp: bool,
    pub fixed_strings: bool,
    pub extended_regexp: bool,
}

impl Default for RegexConfig {
    fn default() -> Self {
        Self {
            ignore_case: false,
            word_regexp: false,
            line_regexp: false,
            fixed_strings: false,
            extended_regexp: false,  // Basic regex is the default, like grep
        }
    }
}
