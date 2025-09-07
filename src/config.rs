use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database file path
    pub database_path: PathBuf,
    
    /// Log level
    pub log_level: String,
    
    /// Maximum number of concurrent parsing tasks
    pub max_concurrent_tasks: usize,
    
    /// Memory limit for indexing operations (in MB)
    pub memory_limit_mb: usize,
    
    /// File extensions to index
    pub cpp_extensions: Vec<String>,
    
    /// Directories to ignore during indexing
    pub ignore_patterns: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("./cpp-index.db"),
            log_level: "info".to_string(),
            max_concurrent_tasks: num_cpus::get(),
            memory_limit_mb: 1024,
            cpp_extensions: vec![
                ".cpp".to_string(),
                ".cc".to_string(),
                ".cxx".to_string(),
                ".c++".to_string(),
                ".c".to_string(),
                ".h".to_string(),
                ".hpp".to_string(),
                ".hh".to_string(),
                ".hxx".to_string(),
                ".h++".to_string(),
            ],
            ignore_patterns: vec![
                "build/".to_string(),
                "target/".to_string(),
                ".git/".to_string(),
                "node_modules/".to_string(),
                "*.o".to_string(),
                "*.obj".to_string(),
                "*.so".to_string(),
                "*.dll".to_string(),
                "*.dylib".to_string(),
            ],
        }
    }
}

impl Config {
    /// Load configuration from file or create default
    #[allow(dead_code)]
    pub fn load() -> Result<Self> {
        // TODO: Implement configuration loading from file
        Ok(Self::default())
    }
    
    /// Save configuration to file
    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        // TODO: Implement configuration saving
        Ok(())
    }
}