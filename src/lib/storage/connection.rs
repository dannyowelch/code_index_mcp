use rusqlite::{Connection, OpenFlags, Result};
use std::path::{Path, PathBuf};
use std::fs;
use crate::lib::storage::schema::{SchemaMigrator, CURRENT_SCHEMA_VERSION};

/// Database configuration options
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to the SQLite database file
    pub database_path: PathBuf,
    /// Whether to create the database if it doesn't exist
    pub create_if_missing: bool,
    /// Whether to enable WAL mode for better concurrency
    pub enable_wal_mode: bool,
    /// Connection pool size (for future connection pooling)
    pub pool_size: u32,
    /// Query timeout in seconds
    pub query_timeout_seconds: u64,
    /// Maximum database size in MB (0 = unlimited)
    pub max_size_mb: u64,
    /// Enable query logging for debugging
    pub enable_query_logging: bool,
}

impl DatabaseConfig {
    /// Creates a new database configuration with default values
    pub fn new<P: AsRef<Path>>(database_path: P) -> Self {
        Self {
            database_path: database_path.as_ref().to_path_buf(),
            create_if_missing: true,
            enable_wal_mode: true,
            pool_size: 10,
            query_timeout_seconds: 30,
            max_size_mb: 0, // Unlimited
            enable_query_logging: false,
        }
    }

    /// Creates configuration for an in-memory database (testing)
    pub fn in_memory() -> Self {
        Self {
            database_path: PathBuf::from(":memory:"),
            create_if_missing: true,
            enable_wal_mode: false, // WAL mode not supported for in-memory databases
            pool_size: 1,
            query_timeout_seconds: 10,
            max_size_mb: 0,
            enable_query_logging: true,
        }
    }

    /// Creates configuration for a temporary database (testing)
    pub fn temporary() -> Result<Self> {
        let temp_dir = std::env::temp_dir();
        let db_name = format!("cpp_index_test_{}.db", uuid::Uuid::new_v4());
        let db_path = temp_dir.join(db_name);
        
        Ok(Self {
            database_path: db_path,
            create_if_missing: true,
            enable_wal_mode: true,
            pool_size: 1,
            query_timeout_seconds: 10,
            max_size_mb: 100, // 100MB limit for temp databases
            enable_query_logging: true,
        })
    }

    /// Sets whether to enable WAL mode
    pub fn with_wal_mode(mut self, enable: bool) -> Self {
        self.enable_wal_mode = enable;
        self
    }

    /// Sets the query timeout
    pub fn with_query_timeout(mut self, timeout_seconds: u64) -> Self {
        self.query_timeout_seconds = timeout_seconds;
        self
    }

    /// Sets the maximum database size
    pub fn with_max_size_mb(mut self, max_size_mb: u64) -> Self {
        self.max_size_mb = max_size_mb;
        self
    }

    /// Enables query logging
    pub fn with_query_logging(mut self, enable: bool) -> Self {
        self.enable_query_logging = enable;
        self
    }

    /// Validates the database configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check if parent directory exists (for file-based databases)
        if self.database_path != Path::new(":memory:") {
            if let Some(parent) = self.database_path.parent() {
                if !parent.exists() && !self.create_if_missing {
                    return Err(format!("Database directory does not exist: {}", parent.display()));
                }
            }
        }

        if self.query_timeout_seconds == 0 {
            return Err("Query timeout must be greater than 0".to_string());
        }

        if self.pool_size == 0 {
            return Err("Pool size must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Returns true if this is an in-memory database
    pub fn is_in_memory(&self) -> bool {
        self.database_path == Path::new(":memory:")
    }

    /// Returns the database file path as a string
    pub fn database_path_str(&self) -> &str {
        self.database_path.to_str().unwrap_or(":memory:")
    }
}

/// Database connection manager that handles connection setup and configuration
pub struct DatabaseManager {
    config: DatabaseConfig,
}

impl DatabaseManager {
    /// Creates a new database manager with the given configuration
    pub fn new(config: DatabaseConfig) -> Result<Self, String> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Opens a connection to the database and applies all migrations
    pub fn connect(&self) -> Result<Connection> {
        self.ensure_database_directory()?;
        let mut connection = self.open_connection()?;
        self.configure_connection(&mut connection)?;
        self.apply_migrations(&mut connection)?;
        Ok(connection)
    }

    /// Opens a connection without applying migrations (for migration testing)
    pub fn connect_raw(&self) -> Result<Connection> {
        self.ensure_database_directory()?;
        let mut connection = self.open_connection()?;
        self.configure_connection(&mut connection)?;
        Ok(connection)
    }

    /// Ensures the database directory exists
    fn ensure_database_directory(&self) -> Result<()> {
        if self.config.is_in_memory() {
            return Ok(());
        }

        if let Some(parent) = self.config.database_path.parent() {
            if !parent.exists() && self.config.create_if_missing {
                fs::create_dir_all(parent)
                    .map_err(|e| rusqlite::Error::SqliteFailure(
                        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                        Some(format!("Failed to create database directory: {}", e)),
                    ))?;
            }
        }

        Ok(())
    }

    /// Opens the SQLite connection with appropriate flags
    fn open_connection(&self) -> Result<Connection> {
        let mut flags = OpenFlags::SQLITE_OPEN_READ_WRITE;
        
        if self.config.create_if_missing {
            flags |= OpenFlags::SQLITE_OPEN_CREATE;
        }

        // For better safety
        flags |= OpenFlags::SQLITE_OPEN_NO_MUTEX;

        Connection::open_with_flags(&self.config.database_path, flags)
    }

    /// Configures the connection with performance and safety settings
    fn configure_connection(&self, connection: &mut Connection) -> Result<()> {
        // Enable foreign key constraints
        connection.execute("PRAGMA foreign_keys = ON", [])?;

        // Configure WAL mode for better concurrency (if enabled and not in-memory)
        if self.config.enable_wal_mode && !self.config.is_in_memory() {
            connection.execute("PRAGMA journal_mode = WAL", [])?;
        }

        // Configure synchronous mode for better performance while maintaining safety
        if self.config.is_in_memory() {
            connection.execute("PRAGMA synchronous = OFF", [])?;
        } else {
            connection.execute("PRAGMA synchronous = NORMAL", [])?;
        }

        // Set cache size (negative value means KB)
        connection.execute("PRAGMA cache_size = -64000", [])?; // 64MB cache

        // Configure temp store in memory for better performance
        connection.execute("PRAGMA temp_store = MEMORY", [])?;

        // Set busy timeout
        connection.busy_timeout(std::time::Duration::from_secs(self.config.query_timeout_seconds))?;

        // Configure page size for better performance (must be done before any tables are created)
        connection.execute("PRAGMA page_size = 4096", [])?;

        // Set maximum database size if specified
        if self.config.max_size_mb > 0 && !self.config.is_in_memory() {
            let max_pages = (self.config.max_size_mb * 1024 * 1024) / 4096; // 4KB pages
            connection.execute(&format!("PRAGMA max_page_count = {}", max_pages), [])?;
        }

        // Enable query optimization
        connection.execute("PRAGMA optimize", [])?;

        Ok(())
    }

    /// Applies all database migrations
    fn apply_migrations(&self, connection: &mut Connection) -> Result<()> {
        let migrated_conn = std::mem::replace(connection, Connection::open(":memory:")?);
        let mut migrator = SchemaMigrator::new(migrated_conn);
        migrator.migrate()?;
        *connection = migrator.into_connection();
        Ok(())
    }

    /// Returns the database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Checks if the database exists and is accessible
    pub fn database_exists(&self) -> bool {
        if self.config.is_in_memory() {
            return true;
        }
        
        self.config.database_path.exists()
    }

    /// Returns database information and statistics
    pub fn get_database_info(&self) -> Result<DatabaseInfo> {
        let connection = self.connect()?;
        
        let schema_version: i32 = connection.query_row(
            "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let page_count: i64 = connection.query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let page_size: i64 = connection.query_row("PRAGMA page_size", [], |row| row.get(0))?;
        let file_size = page_count * page_size;

        let user_version: i32 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        let journal_mode: String = connection.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
        let foreign_keys: bool = connection.query_row("PRAGMA foreign_keys", [], |row| {
            let val: i32 = row.get(0)?;
            Ok(val == 1)
        })?;

        // Count tables
        let table_count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )?;

        // Count indices
        let index_count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )?;

        Ok(DatabaseInfo {
            schema_version,
            file_size_bytes: file_size,
            page_count,
            page_size,
            table_count,
            index_count,
            journal_mode,
            foreign_keys_enabled: foreign_keys,
            user_version,
            database_path: self.config.database_path.clone(),
        })
    }

    /// Performs database maintenance operations
    pub fn maintenance(&self) -> Result<MaintenanceResult> {
        let connection = self.connect()?;
        
        // Run ANALYZE to update query planner statistics
        let analyze_start = std::time::Instant::now();
        connection.execute("ANALYZE", [])?;
        let analyze_duration = analyze_start.elapsed();

        // Run VACUUM if not in WAL mode (VACUUM is not compatible with WAL)
        let vacuum_duration = if !self.config.enable_wal_mode && !self.config.is_in_memory() {
            let vacuum_start = std::time::Instant::now();
            connection.execute("VACUUM", [])?;
            Some(vacuum_start.elapsed())
        } else {
            None
        };

        // Run OPTIMIZE
        let optimize_start = std::time::Instant::now();
        connection.execute("PRAGMA optimize", [])?;
        let optimize_duration = optimize_start.elapsed();

        Ok(MaintenanceResult {
            analyze_duration,
            vacuum_duration,
            optimize_duration,
        })
    }

    /// Deletes the database file (if it's not in-memory)
    pub fn delete_database(&self) -> Result<()> {
        if self.config.is_in_memory() {
            return Ok(());
        }

        if self.database_exists() {
            fs::remove_file(&self.config.database_path)
                .map_err(|e| rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_IOERR_DELETE),
                    Some(format!("Failed to delete database file: {}", e)),
                ))?;
        }

        // Also remove WAL and SHM files if they exist
        let wal_path = self.config.database_path.with_extension("db-wal");
        if wal_path.exists() {
            let _ = fs::remove_file(wal_path);
        }

        let shm_path = self.config.database_path.with_extension("db-shm");
        if shm_path.exists() {
            let _ = fs::remove_file(shm_path);
        }

        Ok(())
    }
}

/// Information about the database
#[derive(Debug, Clone)]
pub struct DatabaseInfo {
    pub schema_version: i32,
    pub file_size_bytes: i64,
    pub page_count: i64,
    pub page_size: i64,
    pub table_count: i64,
    pub index_count: i64,
    pub journal_mode: String,
    pub foreign_keys_enabled: bool,
    pub user_version: i32,
    pub database_path: PathBuf,
}

impl DatabaseInfo {
    /// Returns the file size in a human-readable format
    pub fn file_size_human_readable(&self) -> String {
        let size = self.file_size_bytes as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Returns true if the database schema is up to date
    pub fn is_schema_current(&self) -> bool {
        self.schema_version == CURRENT_SCHEMA_VERSION
    }
}

/// Result of database maintenance operations
#[derive(Debug, Clone)]
pub struct MaintenanceResult {
    pub analyze_duration: std::time::Duration,
    pub vacuum_duration: Option<std::time::Duration>,
    pub optimize_duration: std::time::Duration,
}

impl MaintenanceResult {
    /// Returns the total maintenance duration
    pub fn total_duration(&self) -> std::time::Duration {
        self.analyze_duration + self.vacuum_duration.unwrap_or_default() + self.optimize_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_config_new() {
        let db_path = "/tmp/test.db";
        let config = DatabaseConfig::new(db_path);
        
        assert_eq!(config.database_path, PathBuf::from(db_path));
        assert!(config.create_if_missing);
        assert!(config.enable_wal_mode);
        assert_eq!(config.pool_size, 10);
    }

    #[test]
    fn test_database_config_in_memory() {
        let config = DatabaseConfig::in_memory();
        
        assert!(config.is_in_memory());
        assert!(!config.enable_wal_mode);
        assert_eq!(config.pool_size, 1);
        assert!(config.enable_query_logging);
    }

    #[test]
    fn test_database_config_temporary() {
        let config = DatabaseConfig::temporary().unwrap();
        
        assert!(!config.is_in_memory());
        assert!(config.database_path.to_string_lossy().contains("cpp_index_test_"));
        assert_eq!(config.max_size_mb, 100);
    }

    #[test]
    fn test_database_config_validation() {
        let mut config = DatabaseConfig::in_memory();
        assert!(config.validate().is_ok());

        config.query_timeout_seconds = 0;
        assert!(config.validate().is_err());

        config.query_timeout_seconds = 10;
        config.pool_size = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_database_manager_in_memory() {
        let config = DatabaseConfig::in_memory();
        let manager = DatabaseManager::new(config).unwrap();
        
        let connection = manager.connect().unwrap();
        
        // Verify foreign keys are enabled
        let foreign_keys: i32 = connection.query_row("PRAGMA foreign_keys", [], |row| row.get(0)).unwrap();
        assert_eq!(foreign_keys, 1);
    }

    #[test]
    fn test_database_manager_file_based() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = DatabaseConfig::new(&db_path);
        let manager = DatabaseManager::new(config).unwrap();
        
        assert!(!manager.database_exists());
        
        let _connection = manager.connect().unwrap();
        
        assert!(manager.database_exists());
        assert!(db_path.exists());
    }

    #[test]
    fn test_database_info() {
        let config = DatabaseConfig::in_memory();
        let manager = DatabaseManager::new(config).unwrap();
        let _connection = manager.connect().unwrap();
        
        let info = manager.get_database_info().unwrap();
        
        assert!(info.is_schema_current());
        assert!(info.foreign_keys_enabled);
        assert!(info.table_count > 0);
        assert!(info.index_count > 0);
        assert_eq!(info.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_database_maintenance() {
        let config = DatabaseConfig::in_memory();
        let manager = DatabaseManager::new(config).unwrap();
        let _connection = manager.connect().unwrap();
        
        let result = manager.maintenance().unwrap();
        
        assert!(result.analyze_duration.as_millis() >= 0);
        assert!(result.optimize_duration.as_millis() >= 0);
        assert!(result.vacuum_duration.is_none()); // No vacuum for in-memory
    }

    #[test]
    fn test_file_size_human_readable() {
        let info = DatabaseInfo {
            schema_version: 1,
            file_size_bytes: 1024,
            page_count: 1,
            page_size: 1024,
            table_count: 1,
            index_count: 1,
            journal_mode: "memory".to_string(),
            foreign_keys_enabled: true,
            user_version: 0,
            database_path: PathBuf::from(":memory:"),
        };
        
        assert_eq!(info.file_size_human_readable(), "1.0 KB");
    }

    #[test]
    fn test_database_deletion() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_delete.db");
        let config = DatabaseConfig::new(&db_path);
        let manager = DatabaseManager::new(config).unwrap();
        
        // Create database
        let _connection = manager.connect().unwrap();
        assert!(manager.database_exists());
        
        // Delete database
        manager.delete_database().unwrap();
        assert!(!manager.database_exists());
    }
}