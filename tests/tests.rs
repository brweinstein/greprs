#[cfg(test)]
mod tests {
    use greprs::utils::build_regex;

    #[test]
    fn test_case_sensitive() {
        let re = build_regex("Hello", false).unwrap();
        assert!(re.is_match("Hello world"));
        assert!(!re.is_match("hello world"));
    }

    #[test]
    fn test_case_insensitive() {
        let re = build_regex("Hello", true).unwrap();
        assert!(re.is_match("hello world"));
    }

    #[test]
    fn test_invalid_regex() {
        assert!(build_regex("[invalid", false).is_err());
    }
}
