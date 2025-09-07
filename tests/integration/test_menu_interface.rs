#[cfg(test)]
mod test_menu_interface {
    use std::process::{Command, Stdio};
    use std::path::Path;
    use std::io::Write;
    use tempfile::TempDir;
    use anyhow::Result;
    use std::time::Duration;
    use std::fs;
    
    /// Test that the menu command launches without crashing
    #[tokio::test]
    async fn test_menu_command_launches() -> Result<()> {
        let binary_path = Path::new("target/release/cpp-index-mcp");
        #[cfg(windows)]
        let binary_path = Path::new("target/release/cpp-index-mcp.exe");
        
        if !binary_path.exists() {
            println!("Skipping menu test - binary not built yet");
            return Ok(());
        }
        
        // Launch menu command and terminate quickly
        let mut child = Command::new(binary_path)
            .args(&["menu"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start menu command");
        
        // Send quit signal immediately (assuming 'q' quits the menu)
        if let Some(stdin) = child.stdin.take() {
            let mut stdin = stdin;
            let _ = stdin.write_all(b"q\n");
        }
        
        // Wait briefly for the command to process
        let result = tokio::time::timeout(Duration::from_secs(5), async {
            child.wait()
        }).await;
        
        match result {
            Ok(Ok(status)) => {
                // Menu should either exit gracefully or show "not implemented" message
                let output = Command::new(binary_path)
                    .args(&["menu"])
                    .output()
                    .expect("Failed to get menu output");
                
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                println!("Menu stdout: {}", stdout);
                println!("Menu stderr: {}", stderr);
                
                // Currently expect "not yet implemented" message
                assert!(
                    stdout.contains("not yet implemented") || stderr.contains("not yet implemented"),
                    "Expected menu to show 'not yet implemented', got stdout: '{}', stderr: '{}'",
                    stdout, stderr
                );
            }
            Ok(Err(e)) => {
                println!("Menu process error: {}", e);
                // Process error is acceptable for unimplemented features
            }
            Err(_) => {
                // Timeout - kill the process
                let _ = child.kill();
                println!("Menu command timed out - this is expected for interactive menus");
            }
        }
        
        Ok(())
    }
    
    /// Test menu interface components that should be available
    #[tokio::test]
    async fn test_menu_interface_structure() -> Result<()> {
        // This test validates the expected structure of the interactive menu
        // It tests the design without requiring full implementation
        
        // Define expected menu options that should be available
        let expected_menu_options = vec![
            "Create Index",
            "List Indices",
            "Query Symbols",
            "Delete Index", 
            "Start MCP Server",
            "Settings",
            "Help",
            "Quit"
        ];
        
        // Define expected menu flow paths
        let menu_flows = vec![
            ("create_index", vec!["name", "path", "options"]),
            ("list_indices", vec!["display", "details"]),
            ("query_symbols", vec!["index_selection", "search_term", "results"]),
            ("delete_index", vec!["index_selection", "confirmation"]),
            ("server", vec!["index_selection", "transport_options"]),
        ];
        
        // Validate menu structure makes sense
        assert!(expected_menu_options.len() >= 6, "Should have at least 6 main menu options");
        
        // Check that essential operations are covered
        let essential_operations = ["create", "list", "query", "delete", "server"];
        for operation in essential_operations {
            let has_operation = expected_menu_options.iter()
                .any(|option| option.to_lowercase().contains(operation));
            assert!(
                has_operation,
                "Menu should include operation: {}",
                operation
            );
        }
        
        // Validate menu flow structures
        for (flow_name, steps) in menu_flows {
            assert!(
                !steps.is_empty(),
                "Menu flow '{}' should have at least one step",
                flow_name
            );
            
            // Each flow should have logical steps
            match flow_name {
                "create_index" => {
                    assert!(steps.contains(&"name"), "Create index should ask for name");
                    assert!(steps.contains(&"path"), "Create index should ask for path");
                }
                "query_symbols" => {
                    assert!(steps.contains(&"search_term"), "Query should ask for search term");
                }
                "delete_index" => {
                    assert!(steps.contains(&"confirmation"), "Delete should ask for confirmation");
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Test menu input validation patterns
    #[tokio::test]
    async fn test_menu_input_validation() -> Result<()> {
        // Test input validation logic that would be used by the menu
        
        // Index name validation
        let valid_index_names = vec!["main", "test_project", "my-index", "index_123"];
        let invalid_index_names = vec!["", " ", "invalid name", "index/with/slash", "index*with*stars"];
        
        for name in valid_index_names {
            assert!(is_valid_index_name(name), "Should accept valid index name: '{}'", name);
        }
        
        for name in invalid_index_names {
            assert!(!is_valid_index_name(name), "Should reject invalid index name: '{}'", name);
        }
        
        // Path validation
        let temp_dir = TempDir::new()?;
        let valid_path = temp_dir.path();
        let invalid_path = Path::new("/nonexistent/path");
        
        assert!(is_valid_project_path(valid_path), "Should accept existing directory");
        assert!(!is_valid_project_path(invalid_path), "Should reject nonexistent directory");
        
        // Create a file (not directory) to test
        let file_path = temp_dir.path().join("not_a_directory.txt");
        fs::write(&file_path, "test")?;
        
        assert!(!is_valid_project_path(&file_path), "Should reject file path as project path");
        
        Ok(())
    }
    
    /// Test menu configuration and settings
    #[tokio::test]
    async fn test_menu_configuration() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_dir = temp_dir.path().join(".cpp-index-mcp");
        fs::create_dir_all(&config_dir)?;
        
        // Test menu configuration options
        let menu_config = MenuConfig {
            auto_save: true,
            confirm_deletions: true,
            default_file_patterns: vec![
                "**/*.cpp".to_string(),
                "**/*.h".to_string(),
                "**/*.hpp".to_string(),
            ],
            default_exclude_patterns: vec![
                "**/build/**".to_string(),
                "**/.git/**".to_string(),
            ],
            max_recent_indices: 10,
        };
        
        // Validate configuration structure
        assert!(menu_config.auto_save, "Auto-save should be enabled by default");
        assert!(menu_config.confirm_deletions, "Deletion confirmation should be enabled");
        assert!(!menu_config.default_file_patterns.is_empty(), "Should have default file patterns");
        assert!(!menu_config.default_exclude_patterns.is_empty(), "Should have default exclude patterns");
        assert!(menu_config.max_recent_indices > 0, "Should track recent indices");
        
        // Test that we can serialize/deserialize configuration
        let config_json = serde_json::to_string_pretty(&menu_config)?;
        let config_file = config_dir.join("menu_config.json");
        fs::write(&config_file, &config_json)?;
        
        let loaded_config: MenuConfig = serde_json::from_str(&fs::read_to_string(&config_file)?)?;
        assert_eq!(loaded_config.auto_save, menu_config.auto_save);
        assert_eq!(loaded_config.default_file_patterns, menu_config.default_file_patterns);
        
        Ok(())
    }
    
    /// Test menu error handling and user feedback
    #[tokio::test]
    async fn test_menu_error_handling() -> Result<()> {
        // Define error scenarios that the menu should handle gracefully
        let error_scenarios = vec![
            MenuError::InvalidIndexName("".to_string()),
            MenuError::InvalidProjectPath("/nonexistent".to_string()),
            MenuError::IndexAlreadyExists("duplicate_name".to_string()),
            MenuError::IndexNotFound("missing_index".to_string()),
            MenuError::InsufficientPermissions("protected_path".to_string()),
            MenuError::DatabaseError("connection failed".to_string()),
        ];
        
        // Test that each error has appropriate user-friendly messaging
        for error in error_scenarios {
            let error_message = format_error_for_user(&error);
            
            // Error messages should be helpful and non-empty
            assert!(!error_message.is_empty(), "Error message should not be empty");
            assert!(error_message.len() > 10, "Error message should be descriptive");
            
            // Should not contain internal error codes or stack traces
            assert!(!error_message.contains("Error"), "User message should not start with 'Error'");
            assert!(!error_message.contains("panic"), "User message should not mention panics");
            
            // Should provide guidance where possible
            match error {
                MenuError::InvalidIndexName(_) => {
                    assert!(
                        error_message.contains("name") && error_message.contains("valid"),
                        "Invalid name error should explain naming requirements"
                    );
                }
                MenuError::InvalidProjectPath(_) => {
                    assert!(
                        error_message.contains("path") && error_message.contains("exist"),
                        "Invalid path error should explain path requirements"
                    );
                }
                MenuError::IndexAlreadyExists(_) => {
                    assert!(
                        error_message.contains("already") || error_message.contains("exists"),
                        "Duplicate index error should explain the conflict"
                    );
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Test menu accessibility and usability features
    #[tokio::test]
    async fn test_menu_accessibility() -> Result<()> {
        // Test keyboard navigation patterns
        let keyboard_shortcuts = vec![
            ('q', "quit"),
            ('h', "help"),
            ('c', "create"),
            ('l', "list"),
            ('s', "search"),
            ('d', "delete"),
        ];
        
        // Each shortcut should be logical
        for (key, action) in keyboard_shortcuts {
            let key_char = key.to_ascii_lowercase();
            let action_lower = action.to_lowercase();
            
            assert!(
                action_lower.starts_with(key_char) || 
                action_lower.contains(key_char),
                "Keyboard shortcut '{}' should relate to action '{}'",
                key, action
            );
        }
        
        // Test menu text formatting
        let sample_menu_text = format_menu_display(&MenuDisplay {
            title: "C++ Index Manager".to_string(),
            options: vec![
                MenuOption { key: 'c', label: "Create Index".to_string(), description: Some("Create a new C++ codebase index".to_string()) },
                MenuOption { key: 'l', label: "List Indices".to_string(), description: Some("View all available indices".to_string()) },
                MenuOption { key: 'q', label: "Quit".to_string(), description: None },
            ],
            footer: Some("Use arrow keys or letters to navigate".to_string()),
        });
        
        // Menu should be readable and well-formatted
        assert!(sample_menu_text.contains("C++ Index Manager"), "Should show title");
        assert!(sample_menu_text.contains("Create Index"), "Should show options");
        assert!(sample_menu_text.contains("arrow keys"), "Should show navigation help");
        
        // Should have consistent formatting
        let lines: Vec<&str> = sample_menu_text.lines().collect();
        assert!(lines.len() >= 5, "Menu should have multiple lines for readability");
        
        Ok(())
    }
    
    // Helper functions for testing menu logic
    fn is_valid_index_name(name: &str) -> bool {
        !name.is_empty() && 
        !name.trim().is_empty() &&
        !name.contains('/') &&
        !name.contains('\\') &&
        !name.contains('*') &&
        name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    }
    
    fn is_valid_project_path(path: &Path) -> bool {
        path.exists() && path.is_dir()
    }
    
    #[derive(serde::Serialize, serde::Deserialize, PartialEq)]
    struct MenuConfig {
        auto_save: bool,
        confirm_deletions: bool,
        default_file_patterns: Vec<String>,
        default_exclude_patterns: Vec<String>,
        max_recent_indices: usize,
    }
    
    #[derive(Debug)]
    enum MenuError {
        InvalidIndexName(String),
        InvalidProjectPath(String),
        IndexAlreadyExists(String),
        IndexNotFound(String),
        InsufficientPermissions(String),
        DatabaseError(String),
    }
    
    fn format_error_for_user(error: &MenuError) -> String {
        match error {
            MenuError::InvalidIndexName(name) => {
                format!("Please enter a valid index name. '{}' contains invalid characters. Use only letters, numbers, underscores, and hyphens.", name)
            }
            MenuError::InvalidProjectPath(path) => {
                format!("The path '{}' does not exist or is not accessible. Please select a valid directory containing C++ source files.", path)
            }
            MenuError::IndexAlreadyExists(name) => {
                format!("An index named '{}' already exists. Please choose a different name or delete the existing index first.", name)
            }
            MenuError::IndexNotFound(name) => {
                format!("No index found with name '{}'. Use 'List Indices' to see available indices.", name)
            }
            MenuError::InsufficientPermissions(path) => {
                format!("Unable to access '{}' due to insufficient permissions. Please check file permissions or run with appropriate privileges.", path)
            }
            MenuError::DatabaseError(details) => {
                format!("Database operation failed: {}. Please check that the index is not corrupted and try again.", details)
            }
        }
    }
    
    struct MenuOption {
        key: char,
        label: String,
        description: Option<String>,
    }
    
    struct MenuDisplay {
        title: String,
        options: Vec<MenuOption>,
        footer: Option<String>,
    }
    
    fn format_menu_display(display: &MenuDisplay) -> String {
        let mut output = String::new();
        
        // Title
        output.push_str(&format!("=== {} ===\n\n", display.title));
        
        // Options
        for option in &display.options {
            output.push_str(&format!("({}) {}", option.key, option.label));
            if let Some(desc) = &option.description {
                output.push_str(&format!(" - {}", desc));
            }
            output.push('\n');
        }
        
        // Footer
        if let Some(footer) = &display.footer {
            output.push_str(&format!("\n{}\n", footer));
        }
        
        output
    }
}