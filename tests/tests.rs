#[cfg(test)]
mod tests {
    use greprs::{
        utils::{build_regex, RegexConfig},
        search::{SearchConfig, visit_path}
    };
    use std::fs::{self, File};
    use std::io::{self, Write};
    use tempfile;
    use glob::Pattern as GlobPattern;

    // First, let's fix the simple regex tests
    #[test]
    fn test_case_sensitive() {
        let config = RegexConfig {
            ignore_case: false,
            ..RegexConfig::default()
        };
        let re = build_regex("Hello", &config).unwrap();
        assert!(re.is_match("Hello world"));
        assert!(!re.is_match("hello world"));
    }

    #[test]
    fn test_case_insensitive() {
        let config = RegexConfig {
            ignore_case: true,
            ..RegexConfig::default()
        };
        let re = build_regex("Hello", &config).unwrap();
        assert!(re.is_match("hello world"));
        assert!(re.is_match("HELLO world"));
    }

    #[test]
    fn test_invalid_regex() {
        let config = RegexConfig::default();
        assert!(build_regex("[invalid", &config).is_err());
    }

    #[test]
    fn test_regex_configurations() {
        let test_cases = vec![
            // (pattern, text, config, should_match)
            ("hello", "hello world", RegexConfig {
                ignore_case: true,
                ..RegexConfig::default()
            }, true),
            ("Hello", "hello world", RegexConfig::default(), false),
            ("cat", "concatenate", RegexConfig {
                word_regexp: true,
                ..RegexConfig::default()
            }, false),
            ("cat", "a cat!", RegexConfig {
                word_regexp: true,
                ..RegexConfig::default()
            }, true),
            ("hello world", "hello world!", RegexConfig {
                line_regexp: true,
                ..RegexConfig::default()
            }, false),
            ("hello world", "hello world", RegexConfig {
                line_regexp: true,
                ..RegexConfig::default()
            }, true),
            ("a.c", "abc", RegexConfig {
                fixed_strings: true,
                ..RegexConfig::default()
            }, false),
            ("a.c", "a.c", RegexConfig {
                fixed_strings: true,
                ..RegexConfig::default()
            }, true),
        ];

        for (pattern, text, config, should_match) in test_cases {
            let re = build_regex(pattern, &config).unwrap();
            assert_eq!(re.is_match(text), should_match,
                "Failed for pattern '{}' with text '{}' (config: {:?})",
                pattern, text, config);
        }
    }

    #[test]
    fn test_basic_file_searching() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let test_file = dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        
        writeln!(file, "Hello World")?;
        writeln!(file, "Another line")?;
        writeln!(file, "hello again")?;
        
        let config = SearchConfig {
            line_number: true,
            with_filename: true,
            ..SearchConfig::default()
        };
        
        let re = build_regex("Hello", &RegexConfig::default()).unwrap();
        let mut output = Vec::new();
        visit_path(&re, &test_file, &config, false, &mut output)?;
        
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("1:Hello World"));
        assert!(!output_str.contains("hello again")); // case sensitive
        
        Ok(())
    }

    #[test]
    fn test_context_lines() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let test_file = dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        
        writeln!(file, "Line 1")?;
        writeln!(file, "Line 2")?;
        writeln!(file, "MATCH HERE")?;
        writeln!(file, "Line 4")?;
        writeln!(file, "Line 5")?;
        
        // Test before context
        let config = SearchConfig {
            before_context: Some(1),
            line_number: true,
            ..SearchConfig::default()
        };
        
        let re = build_regex("MATCH", &RegexConfig::default()).unwrap();
        let mut output = Vec::new();
        visit_path(&re, &test_file, &config, false, &mut output)?;
        
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("Line 2"));
        assert!(output_str.contains("MATCH HERE"));
        
        // Test after context
        let config = SearchConfig {
            after_context: Some(1),
            line_number: true,
            ..SearchConfig::default()
        };
        
        let mut output = Vec::new();
        visit_path(&re, &test_file, &config, false, &mut output)?;
        
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("MATCH HERE"));
        assert!(output_str.contains("Line 4"));
        
        Ok(())
    }

    #[test]
    fn test_only_matching() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let test_file = dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        
        writeln!(file, "This line has ERROR in it")?;
        writeln!(file, "No match here")?;
        writeln!(file, "Multiple ERROR and ERROR words")?;
        
        let config = SearchConfig {
            only_matching: true,
            ..SearchConfig::default()
        };
        
        let re = build_regex("ERROR", &RegexConfig::default()).unwrap();
        let mut output = Vec::new();
        visit_path(&re, &test_file, &config, false, &mut output)?;
        
        let output_str = String::from_utf8_lossy(&output);
        let lines: Vec<&str> = output_str.trim().split('\n').collect();
        
        // Should have 3 matches total (1 + 2)
        assert_eq!(lines.len(), 3);
        assert!(lines.iter().all(|line| line.trim() == "ERROR"));
        
        Ok(())
    }

    #[test]
    fn test_max_count() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let test_file = dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        
        for i in 1..=10 {
            writeln!(file, "Match line {}", i)?;
        }
        
        let config = SearchConfig {
            max_count: Some(3),
            line_number: true,
            ..SearchConfig::default()
        };
        
        let re = build_regex("Match", &RegexConfig::default()).unwrap();
        let mut output = Vec::new();
        visit_path(&re, &test_file, &config, false, &mut output)?;
        
        let output_str = String::from_utf8_lossy(&output);
        let lines: Vec<&str> = output_str.trim().split('\n').filter(|l| !l.is_empty()).collect();
        
        // Should only have 3 matches due to max_count
        assert_eq!(lines.len(), 3);
        
        Ok(())
    }

    #[test]
    fn test_byte_offset() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let test_file = dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        
        writeln!(file, "First line")?;
        writeln!(file, "MATCH line")?;
        
        let config = SearchConfig {
            byte_offset: true,
            line_number: true,
            ..SearchConfig::default()
        };
        
        let re = build_regex("MATCH", &RegexConfig::default()).unwrap();
        let mut output = Vec::new();
        visit_path(&re, &test_file, &config, false, &mut output)?;
        
        let output_str = String::from_utf8_lossy(&output);
        // Should contain byte offset (11 = "First line\n".len())
        assert!(output_str.contains("11:"));
        
        Ok(())
    }

    #[test]
    fn test_directory_recursion() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let sub_dir = dir.path().join("subdir");
        fs::create_dir(&sub_dir)?;
        
        let files = vec![
            (dir.path().join("file1.txt"), "Hello World"),
            (dir.path().join("file2.txt"), "No match here"),
            (sub_dir.join("file3.txt"), "Hello again"),
        ];
        
        for (path, content) in files {
            let mut file = File::create(path)?;
            writeln!(file, "{}", content)?;
        }
        
        let config = SearchConfig {
            files_with_matches: true,
            ..SearchConfig::default()
        };
        
        let re = build_regex("Hello", &RegexConfig::default()).unwrap();
        let mut output = Vec::new();
        
        // Test non-recursive (should only find file1.txt)
        visit_path(&re, dir.path(), &config, false, &mut output)?;
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("file1.txt"));
        assert!(!output_str.contains("file3.txt"));
        
        // Test recursive (should find both file1.txt and file3.txt)
        output.clear();
        visit_path(&re, dir.path(), &config, true, &mut output)?;
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("file1.txt"));
        assert!(output_str.contains("file3.txt"));
        
        Ok(())
    }

    #[test]
    fn test_include_exclude_patterns() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        
        let files = vec![
            ("test.txt", "Hello World"),
            ("test.rs", "Hello Rust"),
            ("data.log", "Hello Log"),
            ("readme.md", "Hello Markdown"),
        ];
        
        for (filename, content) in files {
            let mut file = File::create(dir.path().join(filename))?;
            writeln!(file, "{}", content)?;
        }
        
        // Test include pattern - only .rs files
        let config = SearchConfig {
            include_patterns: vec![GlobPattern::new("*.rs").unwrap()],
            files_with_matches: true,
            ..SearchConfig::default()
        };
        
        let re = build_regex("Hello", &RegexConfig::default()).unwrap();
        let mut output = Vec::new();
        visit_path(&re, dir.path(), &config, true, &mut output)?;
        
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("test.rs"));
        assert!(!output_str.contains("test.txt"));
        assert!(!output_str.contains("data.log"));
        
        // Test exclude pattern - exclude .log files
        let config = SearchConfig {
            exclude_patterns: vec![GlobPattern::new("*.log").unwrap()],
            files_with_matches: true,
            ..SearchConfig::default()
        };
        
        output.clear();
        visit_path(&re, dir.path(), &config, true, &mut output)?;
        
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("test.txt"));
        assert!(output_str.contains("test.rs"));
        assert!(!output_str.contains("data.log"));
        
        Ok(())
    }

    #[test]
    fn test_special_options() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let test_file = dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        
        writeln!(file, "Line 1: Hello")?;
        writeln!(file, "Line 2: World")?;
        writeln!(file, "Line 3: Hello")?;
        
        // Test count option
        let config = SearchConfig {
            count: true,
            ..SearchConfig::default()
        };
        
        let re = build_regex("Hello", &RegexConfig::default()).unwrap();
        let mut output = Vec::new();
        visit_path(&re, &test_file, &config, false, &mut output)?;
        
        let output_str = String::from_utf8_lossy(&output);
        assert_eq!(output_str.trim(), "2");
        
        // Test invert match with count
        let config = SearchConfig {
            invert_match: true,
            count: true,
            ..SearchConfig::default()
        };
        
        output.clear();
        visit_path(&re, &test_file, &config, false, &mut output)?;
        
        let output_str = String::from_utf8_lossy(&output);
        assert_eq!(output_str.trim(), "1"); // Only "Line 2: World" doesn't match
        
        // Test files with matches
        let config = SearchConfig {
            files_with_matches: true,
            ..SearchConfig::default()
        };
        
        output.clear();
        visit_path(&re, &test_file, &config, false, &mut output)?;
        
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("test.txt"));
        
        // Test files without match
        let config = SearchConfig {
            files_without_match: true,
            ..SearchConfig::default()
        };
        
        let re_nomatch = build_regex("NOMATCH", &RegexConfig::default()).unwrap();
        output.clear();
        visit_path(&re_nomatch, &test_file, &config, false, &mut output)?;
        
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("test.txt"));
        
        Ok(())
    }

    #[test]
    fn test_quiet_mode() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let test_file = dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        
        writeln!(file, "Hello World")?;
        writeln!(file, "Another line")?;
        
        let config = SearchConfig {
            quiet: true,
            ..SearchConfig::default()
        };
        
        let re = build_regex("Hello", &RegexConfig::default()).unwrap();
        let mut output = Vec::new();
        visit_path(&re, &test_file, &config, false, &mut output)?;
        
        // Quiet mode should produce no output
        assert!(output.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_null_separators() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let test_file = dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        
        // Write null-separated data
        write!(file, "Hello\0World\0Test\0")?;
        
        let config = SearchConfig {
            null_data: true,
            null: true, // Also use null in output
            ..SearchConfig::default()
        };
        
        let re = build_regex("Hello", &RegexConfig::default()).unwrap();
        let mut output = Vec::new();
        visit_path(&re, &test_file, &config, false, &mut output)?;
        
        // Should find the match and output should contain null character
        assert!(!output.is_empty());
        assert!(output.contains(&0u8)); // Contains null byte
        
        Ok(())
    }
}
