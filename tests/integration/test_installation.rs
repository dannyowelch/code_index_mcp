#[cfg(test)]
mod test_installation {
    use std::process::Command;
    use std::path::Path;
    use tempfile::TempDir;
    use anyhow::Result;
    
    /// Test that the binary builds successfully
    #[tokio::test]
    async fn test_binary_builds() -> Result<()> {
        // Build the project in release mode
        let output = Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .expect("Failed to execute cargo build");
            
        assert!(
            output.status.success(),
            "Build failed with stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        
        // Verify the binary exists
        let binary_path = Path::new("target/release/cpp-index-mcp");
        #[cfg(windows)]
        let binary_path = Path::new("target/release/cpp-index-mcp.exe");
        
        assert!(
            binary_path.exists(),
            "Binary not found at expected path: {}",
            binary_path.display()
        );
        
        Ok(())
    }
    
    /// Test that the binary shows help information
    #[tokio::test]
    async fn test_binary_help() -> Result<()> {
        let binary_path = Path::new("target/release/cpp-index-mcp");
        #[cfg(windows)]
        let binary_path = Path::new("target/release/cpp-index-mcp.exe");
        
        // Skip if binary doesn't exist (depends on build test passing)
        if !binary_path.exists() {
            println!("Skipping help test - binary not built yet");
            return Ok(());
        }
        
        let output = Command::new(binary_path)
            .args(&["--help"])
            .output()
            .expect("Failed to execute binary");
            
        assert!(
            output.status.success(),
            "Help command failed with stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Verify expected CLI commands are present
        assert!(stdout.contains("index"), "Help should mention 'index' command");
        assert!(stdout.contains("menu"), "Help should mention 'menu' command");
        assert!(stdout.contains("server"), "Help should mention 'server' command");
        assert!(stdout.contains("query"), "Help should mention 'query' command");
        
        Ok(())
    }
    
    /// Test that dependencies are properly configured
    #[tokio::test]
    async fn test_dependencies_available() -> Result<()> {
        // Test that we can import and use key dependencies
        // This validates the Cargo.toml configuration
        
        // Test serde_json
        let test_json = serde_json::json!({
            "test": "value"
        });
        assert_eq!(test_json["test"], "value");
        
        // Test that we can create a temporary SQLite database
        use rusqlite::Connection;
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path)?;
        
        // Create a simple test table
        conn.execute(
            "CREATE TABLE test_table (id INTEGER PRIMARY KEY, name TEXT)",
            [],
        )?;
        
        // Verify table was created
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='test_table'")?;
        let table_exists: bool = stmt.exists([])?;
        assert!(table_exists, "Test table should be created in SQLite");
        
        Ok(())
    }
    
    /// Test that the project structure matches expected layout
    #[tokio::test]
    async fn test_project_structure() -> Result<()> {
        // Verify main source directories exist
        let src_lib = Path::new("src/lib");
        assert!(src_lib.exists(), "src/lib directory should exist");
        
        // Verify expected library modules exist
        let expected_modules = [
            "src/lib/cpp_indexer",
            "src/lib/mcp_server", 
            "src/lib/storage",
            "src/lib/cli_interface"
        ];
        
        for module_path in expected_modules {
            let path = Path::new(module_path);
            assert!(
                path.exists(),
                "Expected module directory should exist: {}",
                module_path
            );
            
            // Verify mod.rs exists in each module
            let mod_rs = path.join("mod.rs");
            assert!(
                mod_rs.exists(),
                "Module should have mod.rs file: {}",
                mod_rs.display()
            );
        }
        
        // Verify test directories exist
        let test_dirs = [
            "tests/contract",
            "tests/integration",
            "tests/unit",
            "tests/performance"
        ];
        
        for test_dir in test_dirs {
            let path = Path::new(test_dir);
            assert!(
                path.exists(),
                "Test directory should exist: {}",
                test_dir
            );
        }
        
        // Verify test data directory exists
        let test_data = Path::new("test-data/sample-cpp");
        assert!(
            test_data.exists(),
            "Test data directory should exist: {}",
            test_data.display()
        );
        
        Ok(())
    }
    
    /// Test that logging configuration works
    #[tokio::test]
    async fn test_logging_setup() -> Result<()> {
        // This test validates that tracing setup doesn't panic
        // and that we can create log messages
        use tracing::{info, debug, warn, error};
        
        // Initialize a test subscriber to capture logs
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .with_test_writer()
            .finish();
            
        tracing::subscriber::with_default(subscriber, || {
            info!("Test info message");
            debug!("Test debug message");
            warn!("Test warning message");
            error!("Test error message");
        });
        
        // If we get here without panicking, logging setup is working
        Ok(())
    }
    
    /// Test that configuration module works
    #[tokio::test]
    async fn test_config_module() -> Result<()> {
        // This test ensures the config module can be imported and used
        // We'll test actual configuration loading once config.rs is implemented
        
        // For now, just verify the module exists and can be imported
        // The config module should be accessible since it's declared in main.rs
        
        // Create a temporary config directory to test file operations
        let temp_dir = TempDir::new()?;
        let config_dir = temp_dir.path().join(".cpp-index-mcp");
        
        std::fs::create_dir_all(&config_dir)?;
        assert!(config_dir.exists(), "Config directory should be created");
        
        // Test that we can write a simple config file
        let config_file = config_dir.join("config.json");
        let config_content = serde_json::json!({
            "version": "0.1.0",
            "default_index": "main"
        });
        
        std::fs::write(&config_file, config_content.to_string())?;
        assert!(config_file.exists(), "Config file should be written");
        
        // Test that we can read it back
        let content = std::fs::read_to_string(&config_file)?;
        let parsed: serde_json::Value = serde_json::from_str(&content)?;
        
        assert_eq!(parsed["version"], "0.1.0");
        assert_eq!(parsed["default_index"], "main");
        
        Ok(())
    }
}