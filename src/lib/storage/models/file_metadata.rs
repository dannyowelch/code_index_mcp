use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tracks file-level information for incremental updates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileMetadata {
    /// Unique identifier (auto-increment)
    pub id: Option<i64>,
    /// Foreign key to Code Index
    pub index_id: Uuid,
    /// Relative path from codebase root
    pub file_path: String,
    /// Blake3 hash of entire file content
    pub file_hash: String,
    /// File system modification time
    pub last_modified: DateTime<Utc>,
    /// File size in bytes
    pub size_bytes: u64,
    /// Number of symbols in this file
    pub symbol_count: u32,
    /// Timestamp when file was last indexed
    pub indexed_at: DateTime<Utc>,
}

/// Represents the state of file processing
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileProcessingState {
    /// File is queued for processing
    Pending,
    /// File is currently being processed
    Processing,
    /// File has been successfully indexed
    Indexed,
    /// Error occurred during processing
    Error,
}

impl FileMetadata {
    /// Creates a new FileMetadata
    pub fn new(
        index_id: Uuid,
        file_path: String,
        file_hash: String,
        last_modified: DateTime<Utc>,
        size_bytes: u64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            index_id,
            file_path,
            file_hash,
            last_modified,
            size_bytes,
            symbol_count: 0,
            indexed_at: now,
        }
    }

    /// Updates the symbol count and indexed timestamp
    pub fn update_indexing(&mut self, symbol_count: u32) {
        self.symbol_count = symbol_count;
        self.indexed_at = Utc::now();
    }

    /// Updates the file hash and modification time
    pub fn update_file_info(&mut self, file_hash: String, last_modified: DateTime<Utc>, size_bytes: u64) {
        self.file_hash = file_hash;
        self.last_modified = last_modified;
        self.size_bytes = size_bytes;
    }

    /// Validates the file metadata fields
    pub fn validate(&self) -> Result<(), String> {
        if self.file_path.trim().is_empty() {
            return Err("File path cannot be empty".to_string());
        }

        // Validate file path is relative
        if std::path::Path::new(&self.file_path).is_absolute() {
            return Err("File path must be relative".to_string());
        }

        // Validate Blake3 hash format (64 character hex string)
        if self.file_hash.len() != 64 {
            return Err("File hash must be 64 characters".to_string());
        }

        if !self.file_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("File hash must contain only hexadecimal characters".to_string());
        }

        Ok(())
    }

    /// Checks if the file has been modified since it was last indexed
    pub fn needs_reindexing(&self, current_hash: &str, current_modified: DateTime<Utc>) -> bool {
        self.file_hash != current_hash || self.last_modified < current_modified
    }

    /// Returns the file extension
    pub fn extension(&self) -> Option<&str> {
        std::path::Path::new(&self.file_path)
            .extension()
            .and_then(|ext| ext.to_str())
    }

    /// Returns true if this is a C++ source file
    pub fn is_cpp_source(&self) -> bool {
        matches!(
            self.extension(),
            Some("cpp") | Some("cc") | Some("cxx") | Some("c++") | Some("C")
        )
    }

    /// Returns true if this is a C++ header file
    pub fn is_cpp_header(&self) -> bool {
        matches!(
            self.extension(),
            Some("h") | Some("hpp") | Some("hxx") | Some("h++") | Some("H")
        )
    }

    /// Returns true if this is any C++ file (source or header)
    pub fn is_cpp_file(&self) -> bool {
        self.is_cpp_source() || self.is_cpp_header()
    }

    /// Returns a normalized path with forward slashes
    pub fn normalized_path(&self) -> String {
        self.file_path.replace('\\', "/")
    }

    /// Returns the directory portion of the file path
    pub fn directory(&self) -> Option<&str> {
        std::path::Path::new(&self.file_path)
            .parent()
            .and_then(|p| p.to_str())
    }

    /// Returns the filename portion without directory
    pub fn filename(&self) -> Option<&str> {
        std::path::Path::new(&self.file_path)
            .file_name()
            .and_then(|name| name.to_str())
    }
}

impl FileProcessingState {
    /// Returns true if the file is in a completed state (successfully or with error)
    pub fn is_complete(&self) -> bool {
        matches!(self, FileProcessingState::Indexed | FileProcessingState::Error)
    }

    /// Returns true if the file is currently being processed
    pub fn is_active(&self) -> bool {
        matches!(self, FileProcessingState::Processing)
    }

    /// Returns string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            FileProcessingState::Pending => "pending",
            FileProcessingState::Processing => "processing",
            FileProcessingState::Indexed => "indexed",
            FileProcessingState::Error => "error",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_metadata() -> FileMetadata {
        FileMetadata::new(
            Uuid::new_v4(),
            "src/test.cpp".to_string(),
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
            1024,
        )
    }

    #[test]
    fn test_file_metadata_new() {
        let index_id = Uuid::new_v4();
        let file_path = "include/header.h".to_string();
        let file_hash = "a".repeat(64);
        let modified = Utc::now();
        let size = 2048;

        let metadata = FileMetadata::new(index_id, file_path.clone(), file_hash.clone(), modified, size);

        assert_eq!(metadata.index_id, index_id);
        assert_eq!(metadata.file_path, file_path);
        assert_eq!(metadata.file_hash, file_hash);
        assert_eq!(metadata.last_modified, modified);
        assert_eq!(metadata.size_bytes, size);
        assert_eq!(metadata.symbol_count, 0);
        assert!(metadata.indexed_at <= Utc::now());
    }

    #[test]
    fn test_update_indexing() {
        let mut metadata = create_test_metadata();
        let original_indexed_at = metadata.indexed_at;

        // Sleep a tiny bit to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        metadata.update_indexing(42);

        assert_eq!(metadata.symbol_count, 42);
        assert!(metadata.indexed_at > original_indexed_at);
    }

    #[test]
    fn test_update_file_info() {
        let mut metadata = create_test_metadata();
        let new_hash = "b".repeat(64);
        let new_modified = Utc::now();
        let new_size = 4096;

        metadata.update_file_info(new_hash.clone(), new_modified, new_size);

        assert_eq!(metadata.file_hash, new_hash);
        assert_eq!(metadata.last_modified, new_modified);
        assert_eq!(metadata.size_bytes, new_size);
    }

    #[test]
    fn test_validation() {
        let mut metadata = create_test_metadata();
        assert!(metadata.validate().is_ok());

        // Test empty file path
        metadata.file_path = "".to_string();
        assert!(metadata.validate().is_err());

        // Test absolute file path
        metadata.file_path = if cfg!(windows) {
            "C:\\absolute\\path.cpp".to_string()
        } else {
            "/absolute/path.cpp".to_string()
        };
        assert!(metadata.validate().is_err());

        // Test invalid hash length
        metadata.file_path = "relative/path.cpp".to_string();
        metadata.file_hash = "short".to_string();
        assert!(metadata.validate().is_err());

        // Test invalid hash characters
        metadata.file_hash = "g".repeat(64);
        assert!(metadata.validate().is_err());
    }

    #[test]
    fn test_needs_reindexing() {
        let metadata = create_test_metadata();
        let same_hash = metadata.file_hash.clone();
        let same_time = metadata.last_modified;
        let different_hash = "b".repeat(64);
        let later_time = Utc::now();

        assert!(!metadata.needs_reindexing(&same_hash, same_time));
        assert!(metadata.needs_reindexing(&different_hash, same_time));
        assert!(metadata.needs_reindexing(&same_hash, later_time));
        assert!(metadata.needs_reindexing(&different_hash, later_time));
    }

    #[test]
    fn test_file_type_detection() {
        let mut metadata = create_test_metadata();

        // Test C++ source files
        metadata.file_path = "src/test.cpp".to_string();
        assert!(metadata.is_cpp_source());
        assert!(!metadata.is_cpp_header());
        assert!(metadata.is_cpp_file());

        metadata.file_path = "src/test.cc".to_string();
        assert!(metadata.is_cpp_source());
        assert!(metadata.is_cpp_file());

        // Test C++ header files
        metadata.file_path = "include/test.h".to_string();
        assert!(!metadata.is_cpp_source());
        assert!(metadata.is_cpp_header());
        assert!(metadata.is_cpp_file());

        metadata.file_path = "include/test.hpp".to_string();
        assert!(metadata.is_cpp_header());
        assert!(metadata.is_cpp_file());

        // Test non-C++ files
        metadata.file_path = "README.txt".to_string();
        assert!(!metadata.is_cpp_source());
        assert!(!metadata.is_cpp_header());
        assert!(!metadata.is_cpp_file());
    }

    #[test]
    fn test_path_operations() {
        let metadata = create_test_metadata();
        
        assert_eq!(metadata.extension(), Some("cpp"));
        assert_eq!(metadata.normalized_path(), "src/test.cpp");
        assert_eq!(metadata.directory(), Some("src"));
        assert_eq!(metadata.filename(), Some("test.cpp"));

        // Test Windows path normalization
        let mut metadata = create_test_metadata();
        metadata.file_path = "src\\subdir\\file.h".to_string();
        assert_eq!(metadata.normalized_path(), "src/subdir/file.h");
    }

    #[test]
    fn test_file_processing_state() {
        assert!(FileProcessingState::Indexed.is_complete());
        assert!(FileProcessingState::Error.is_complete());
        assert!(!FileProcessingState::Pending.is_complete());
        assert!(!FileProcessingState::Processing.is_complete());

        assert!(FileProcessingState::Processing.is_active());
        assert!(!FileProcessingState::Pending.is_active());
        assert!(!FileProcessingState::Indexed.is_active());
        assert!(!FileProcessingState::Error.is_active());

        assert_eq!(FileProcessingState::Pending.as_str(), "pending");
        assert_eq!(FileProcessingState::Processing.as_str(), "processing");
        assert_eq!(FileProcessingState::Indexed.as_str(), "indexed");
        assert_eq!(FileProcessingState::Error.as_str(), "error");
    }
}