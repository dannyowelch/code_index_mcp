use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a complete searchable index for a C++ codebase
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeIndex {
    /// Unique identifier (UUID)
    pub id: Uuid,
    /// Human-readable name (e.g., "Unreal Engine 5.3")
    pub name: String,
    /// Root directory path of the indexed codebase
    pub base_path: String,
    /// Timestamp of index creation
    pub created_at: DateTime<Utc>,
    /// Timestamp of last index update
    pub updated_at: DateTime<Utc>,
    /// Count of indexed files
    pub total_files: u32,
    /// Count of indexed symbols
    pub total_symbols: u32,
    /// Schema version for migration support
    pub index_version: u32,
}

/// Represents the state of a Code Index during its lifecycle
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IndexState {
    /// Index is being built for the first time
    Creating,
    /// Index is complete and queryable
    Active,
    /// Incremental update in progress
    Updating,
    /// Index preserved but not actively maintained
    Archived,
    /// Index creation or update failed
    Failed,
}

impl CodeIndex {
    /// Creates a new CodeIndex with the given name and base path
    pub fn new(name: String, base_path: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            base_path,
            created_at: now,
            updated_at: now,
            total_files: 0,
            total_symbols: 0,
            index_version: 1,
        }
    }

    /// Updates the index statistics and timestamps
    pub fn update_stats(&mut self, total_files: u32, total_symbols: u32) {
        self.total_files = total_files;
        self.total_symbols = total_symbols;
        self.updated_at = Utc::now();
    }

    /// Validates the code index fields
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        if self.base_path.trim().is_empty() {
            return Err("Base path cannot be empty".to_string());
        }

        // Validate that base_path is a valid directory path format
        if !std::path::Path::new(&self.base_path).is_absolute() {
            return Err("Base path must be an absolute path".to_string());
        }

        Ok(())
    }
}

impl IndexState {
    /// Returns true if the index is in a state where it can be queried
    pub fn is_queryable(&self) -> bool {
        matches!(self, IndexState::Active)
    }

    /// Returns true if the index is in a state where it can be updated
    pub fn can_update(&self) -> bool {
        matches!(self, IndexState::Active | IndexState::Failed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_index_new() {
        let name = "Test Project".to_string();
        let base_path = "/path/to/project".to_string();
        let index = CodeIndex::new(name.clone(), base_path.clone());

        assert_eq!(index.name, name);
        assert_eq!(index.base_path, base_path);
        assert_eq!(index.total_files, 0);
        assert_eq!(index.total_symbols, 0);
        assert_eq!(index.index_version, 1);
        assert!(index.created_at <= Utc::now());
        assert!(index.updated_at <= Utc::now());
    }

    #[test]
    fn test_update_stats() {
        let mut index = CodeIndex::new("Test".to_string(), "/path".to_string());
        let original_updated = index.updated_at;

        // Sleep a tiny bit to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        index.update_stats(100, 500);

        assert_eq!(index.total_files, 100);
        assert_eq!(index.total_symbols, 500);
        assert!(index.updated_at > original_updated);
    }

    #[test]
    fn test_validation() {
        let mut index = if cfg!(windows) {
            CodeIndex::new("Valid Name".to_string(), "C:\\absolute\\path".to_string())
        } else {
            CodeIndex::new("Valid Name".to_string(), "/absolute/path".to_string())
        };
        assert!(index.validate().is_ok());

        // Test empty name
        index.name = "".to_string();
        assert!(index.validate().is_err());

        // Test empty base path
        index.name = "Valid Name".to_string();
        index.base_path = "".to_string();
        assert!(index.validate().is_err());

        // Test relative path
        index.base_path = "relative/path".to_string();
        assert!(index.validate().is_err());
    }

    #[test]
    fn test_index_state() {
        assert!(IndexState::Active.is_queryable());
        assert!(!IndexState::Creating.is_queryable());
        assert!(!IndexState::Updating.is_queryable());
        assert!(!IndexState::Archived.is_queryable());
        assert!(!IndexState::Failed.is_queryable());

        assert!(IndexState::Active.can_update());
        assert!(IndexState::Failed.can_update());
        assert!(!IndexState::Creating.can_update());
        assert!(!IndexState::Updating.can_update());
        assert!(!IndexState::Archived.can_update());
    }
}