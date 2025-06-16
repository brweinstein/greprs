use regex::RegexBuilder;

pub fn build_regex(pattern: &str, ignore_case: bool) -> Result<regex::Regex, regex::Error> {
    RegexBuilder::new(pattern)
        .case_insensitive(ignore_case)
        .build()
}
