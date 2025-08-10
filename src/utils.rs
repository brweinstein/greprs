use regex::{Regex, RegexBuilder};

#[derive(Debug, Default)]
pub struct RegexConfig {
    pub ignore_case: bool,
    pub word_regexp: bool,
    pub line_regexp: bool,
    pub fixed_strings: bool,
}

pub fn build_regex(pattern: &str, config: &RegexConfig) -> Result<Regex, String> {
    let mut pattern = pattern.to_string();
    
    // Handle different regex types
    if config.fixed_strings {
        pattern = regex::escape(&pattern);
    }
    
    if config.word_regexp {
        pattern = format!(r"\b{}\b", pattern);
    }
    
    if config.line_regexp {
        pattern = format!("^{}$", pattern);
    }
    
    let regex_result = RegexBuilder::new(&pattern)
        .case_insensitive(config.ignore_case)
        .build();
    
    match regex_result {
        Ok(re) => Ok(re),
        Err(e) => Err(format!("Invalid regex pattern: {}", e)),
    }
}
