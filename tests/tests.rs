#[cfg(test)]
mod tests {
    use greprs::{
        utils::{build_regex, RegexConfig},
        search::{SearchConfig, visit_path}
    };
    use std::fs::{self, File};
    use std::io::{self, Write};
    use tempfile;

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
            ("a.c", "abc", RegexConfig {
                fixed_strings: true,
                ..RegexConfig::default()
            }, false),
        ];

        for (pattern, text, config, should_match) in test_cases {
            let re = build_regex(pattern, &config).unwrap();
            assert_eq!(re.is_match(text), should_match,
                "Failed for pattern '{}' with text '{}' (config: {:?})",
                pattern, text, config);
        }
    }

    #[test]
    fn test_file_searching() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let test_file = dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        
        writeln!(file, "Hello World")?;
        writeln!(file, "Another line")?;
        writeln!(file, "hello again")?;
        
        let config = SearchConfig {
            invert_match: false,
            line_number: true,
            with_filename: true,
            no_filename: false,
            count: false,
            files_with_matches: false,
            files_without_match: false,
        };
        
        let re = build_regex("Hello", &RegexConfig::default()).unwrap();
        let mut output = Vec::new();
        visit_path(&re, &test_file, &config, false, &mut output)?;
        
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
            invert_match: false,
            line_number: false,
            with_filename: true,
            no_filename: false,
            count: false,
            files_with_matches: true,
            files_without_match: false,
        };
        
        let re = build_regex("Hello", &RegexConfig::default()).unwrap();
        let mut output = Vec::new();
        
        visit_path(&re, dir.path(), &config, false, &mut output)?;
        visit_path(&re, dir.path(), &config, true, &mut output)?;
        
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
        
        struct TestCase {
            config: SearchConfig,
            expected_count: usize,
            pattern: &'static str,
        }

        let test_cases = vec![
            TestCase {
                config: SearchConfig {
                    count: true,
                    ..SearchConfig::default()
                },
                expected_count: 2,
                pattern: "Hello",
            },
            TestCase {
                config: SearchConfig {
                    invert_match: true,
                    count: true,
                    ..SearchConfig::default()
                },
                expected_count: 1,
                pattern: "Hello",
            },
            TestCase {
                config: SearchConfig {
                    files_with_matches: true,
                    ..SearchConfig::default()
                },
                expected_count: 1,
                pattern: "Hello",
            },
        ];
        
        for test_case in test_cases {
            let regex_config = RegexConfig::default();
            let re = build_regex(test_case.pattern, &regex_config).unwrap();
            
            // Create a buffer to capture output
            let mut output = Vec::new();
            
            // Run the search with captured output
            visit_path(&re, &test_file, &test_case.config, false, &mut output)?;
            
            // Convert output to string and count lines/matches
            let output_str = String::from_utf8_lossy(&output);
            let actual_count = if test_case.config.count {
                output_str.trim().parse::<usize>().unwrap_or(0)
            } else {
                output_str.lines().count()
            };
            
            assert_eq!(
                actual_count,
                test_case.expected_count,
                "Failed for config {:?}: expected {} matches but got {}\nOutput: {:?}",
                test_case.config,
                test_case.expected_count,
                actual_count,
                output_str
            );
        }
        
        Ok(())
    }
}
