use rusqlite::{Connection, Result};
use std::collections::HashMap;

/// Database schema version - increment when making schema changes
pub const CURRENT_SCHEMA_VERSION: i32 = 1;

/// Schema migration manager for SQLite database
pub struct SchemaMigrator {
    connection: Connection,
}

impl SchemaMigrator {
    /// Creates a new schema migrator with the given database connection
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    /// Runs all necessary migrations to bring the database to the current schema version
    pub fn migrate(&mut self) -> Result<()> {
        self.ensure_migration_table()?;
        let current_version = self.get_current_version()?;
        
        if current_version < CURRENT_SCHEMA_VERSION {
            self.run_migrations_from(current_version)?;
            self.set_schema_version(CURRENT_SCHEMA_VERSION)?;
        }
        
        Ok(())
    }

    /// Returns the current schema version of the database
    pub fn get_current_version(&self) -> Result<i32> {
        let version: Result<i32> = self.connection.query_row(
            "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        );
        
        match version {
            Ok(v) => Ok(v),
            Err(rusqlite::Error::SqliteFailure(_, _)) => Ok(0), // No migrations table yet
            Err(e) => Err(e),
        }
    }

    /// Creates the schema_migrations table if it doesn't exist
    fn ensure_migration_table(&self) -> Result<()> {
        self.connection.execute(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )?;
        Ok(())
    }

    /// Records a schema version as applied
    fn set_schema_version(&self, version: i32) -> Result<()> {
        self.connection.execute(
            "INSERT OR REPLACE INTO schema_migrations (version) VALUES (?1)",
            [version],
        )?;
        Ok(())
    }

    /// Runs all migrations starting from the given version
    fn run_migrations_from(&mut self, from_version: i32) -> Result<()> {
        let migrations = self.get_migrations();
        
        for version in (from_version + 1)..=CURRENT_SCHEMA_VERSION {
            if let Some(migration_sql) = migrations.get(&version) {
                self.connection.execute_batch(migration_sql)?;
            }
        }
        
        Ok(())
    }

    /// Returns a map of version -> SQL migration statements
    fn get_migrations(&self) -> HashMap<i32, &'static str> {
        let mut migrations = HashMap::new();
        
        // Migration 1: Initial schema
        migrations.insert(1, MIGRATION_V1);
        
        migrations
    }

    /// Returns a reference to the underlying connection
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Consumes the migrator and returns the connection
    pub fn into_connection(self) -> Connection {
        self.connection
    }
}

/// Migration V1: Initial schema with all core tables
const MIGRATION_V1: &str = r#"
-- Enable foreign key constraints
PRAGMA foreign_keys = ON;

-- Code Index table
CREATE TABLE code_indices (
    id TEXT PRIMARY KEY,  -- UUID as TEXT
    name TEXT NOT NULL,
    base_path TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    total_files INTEGER NOT NULL DEFAULT 0,
    total_symbols INTEGER NOT NULL DEFAULT 0,
    index_version INTEGER NOT NULL DEFAULT 1,
    state TEXT NOT NULL DEFAULT 'creating' CHECK (state IN ('creating', 'active', 'updating', 'archived', 'failed')),
    UNIQUE(name)
);

-- Create index for faster lookups
CREATE INDEX idx_code_indices_name ON code_indices(name);
CREATE INDEX idx_code_indices_state ON code_indices(state);
CREATE INDEX idx_code_indices_updated_at ON code_indices(updated_at);

-- File Metadata table
CREATE TABLE file_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    index_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    last_modified DATETIME NOT NULL,
    size_bytes INTEGER NOT NULL,
    symbol_count INTEGER NOT NULL DEFAULT 0,
    indexed_at DATETIME NOT NULL,
    processing_state TEXT NOT NULL DEFAULT 'pending' CHECK (processing_state IN ('pending', 'processing', 'indexed', 'error')),
    FOREIGN KEY (index_id) REFERENCES code_indices(id) ON DELETE CASCADE,
    UNIQUE(index_id, file_path)
);

-- Create indices for file metadata
CREATE INDEX idx_file_metadata_index_id ON file_metadata(index_id);
CREATE INDEX idx_file_metadata_file_path ON file_metadata(file_path);
CREATE INDEX idx_file_metadata_file_hash ON file_metadata(file_hash);
CREATE INDEX idx_file_metadata_last_modified ON file_metadata(last_modified);
CREATE INDEX idx_file_metadata_processing_state ON file_metadata(processing_state);

-- Code Elements table
CREATE TABLE code_elements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    index_id TEXT NOT NULL,
    symbol_name TEXT NOT NULL,
    symbol_type TEXT NOT NULL CHECK (symbol_type IN ('function', 'class', 'struct', 'variable', 'macro', 'namespace', 'enum', 'typedef', 'union', 'template', 'constructor', 'destructor', 'operator')),
    file_path TEXT NOT NULL,
    line_number INTEGER NOT NULL,
    column_number INTEGER NOT NULL,
    definition_hash TEXT NOT NULL,
    scope TEXT,
    access_modifier TEXT CHECK (access_modifier IN ('public', 'private', 'protected')),
    is_declaration BOOLEAN NOT NULL DEFAULT 0,
    signature TEXT,
    FOREIGN KEY (index_id) REFERENCES code_indices(id) ON DELETE CASCADE
);

-- Create indices for code elements
CREATE INDEX idx_code_elements_index_id ON code_elements(index_id);
CREATE INDEX idx_code_elements_symbol_name ON code_elements(symbol_name);
CREATE INDEX idx_code_elements_symbol_type ON code_elements(symbol_type);
CREATE INDEX idx_code_elements_file_path ON code_elements(file_path);
CREATE INDEX idx_code_elements_scope ON code_elements(scope);
CREATE INDEX idx_code_elements_definition_hash ON code_elements(definition_hash);
CREATE INDEX idx_code_elements_composite ON code_elements(index_id, symbol_name, symbol_type);

-- Symbol Relationships table
CREATE TABLE symbol_relationships (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_symbol_id INTEGER NOT NULL,
    to_symbol_id INTEGER NOT NULL,
    relationship_type TEXT NOT NULL CHECK (relationship_type IN ('inherits', 'uses', 'includes', 'calls', 'defines', 'instantiates', 'contained_in', 'friend', 'overrides', 'specializes')),
    file_path TEXT NOT NULL,
    line_number INTEGER NOT NULL,
    FOREIGN KEY (from_symbol_id) REFERENCES code_elements(id) ON DELETE CASCADE,
    FOREIGN KEY (to_symbol_id) REFERENCES code_elements(id) ON DELETE CASCADE,
    UNIQUE(from_symbol_id, to_symbol_id, relationship_type, line_number)
);

-- Create indices for symbol relationships
CREATE INDEX idx_symbol_relationships_from ON symbol_relationships(from_symbol_id);
CREATE INDEX idx_symbol_relationships_to ON symbol_relationships(to_symbol_id);
CREATE INDEX idx_symbol_relationships_type ON symbol_relationships(relationship_type);
CREATE INDEX idx_symbol_relationships_file_path ON symbol_relationships(file_path);

-- MCP Query Sessions table
CREATE TABLE mcp_query_sessions (
    session_id TEXT PRIMARY KEY,  -- UUID as TEXT
    client_name TEXT NOT NULL,
    active_index_id TEXT,
    created_at DATETIME NOT NULL,
    last_activity DATETIME NOT NULL,
    query_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'terminated', 'error')),
    client_metadata TEXT,  -- JSON string for flexible metadata
    FOREIGN KEY (active_index_id) REFERENCES code_indices(id) ON DELETE SET NULL
);

-- Create indices for MCP query sessions
CREATE INDEX idx_mcp_sessions_client_name ON mcp_query_sessions(client_name);
CREATE INDEX idx_mcp_sessions_status ON mcp_query_sessions(status);
CREATE INDEX idx_mcp_sessions_active_index ON mcp_query_sessions(active_index_id);
CREATE INDEX idx_mcp_sessions_last_activity ON mcp_query_sessions(last_activity);

-- Create a view for commonly used queries
CREATE VIEW symbol_details_view AS
SELECT 
    ce.id,
    ce.symbol_name,
    ce.symbol_type,
    ce.file_path,
    ce.line_number,
    ce.column_number,
    ce.scope,
    ce.access_modifier,
    ce.is_declaration,
    ce.signature,
    ci.name as index_name,
    fm.last_modified as file_last_modified
FROM code_elements ce
JOIN code_indices ci ON ce.index_id = ci.id
LEFT JOIN file_metadata fm ON ce.index_id = fm.index_id AND ce.file_path = fm.file_path
WHERE ci.state = 'active';

-- Create a view for file statistics
CREATE VIEW file_stats_view AS
SELECT 
    fm.index_id,
    ci.name as index_name,
    COUNT(*) as total_files,
    SUM(fm.symbol_count) as total_symbols,
    AVG(fm.size_bytes) as avg_file_size,
    MAX(fm.last_modified) as latest_file_modified
FROM file_metadata fm
JOIN code_indices ci ON fm.index_id = ci.id
WHERE fm.processing_state = 'indexed'
GROUP BY fm.index_id, ci.name;

-- Create triggers for maintaining statistics
CREATE TRIGGER update_index_stats_on_file_insert
AFTER INSERT ON file_metadata
WHEN NEW.processing_state = 'indexed'
BEGIN
    UPDATE code_indices 
    SET total_files = (
        SELECT COUNT(*) 
        FROM file_metadata 
        WHERE index_id = NEW.index_id AND processing_state = 'indexed'
    ),
    total_symbols = (
        SELECT COALESCE(SUM(symbol_count), 0) 
        FROM file_metadata 
        WHERE index_id = NEW.index_id AND processing_state = 'indexed'
    ),
    updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.index_id;
END;

CREATE TRIGGER update_index_stats_on_file_update
AFTER UPDATE ON file_metadata
WHEN NEW.processing_state = 'indexed'
BEGIN
    UPDATE code_indices 
    SET total_files = (
        SELECT COUNT(*) 
        FROM file_metadata 
        WHERE index_id = NEW.index_id AND processing_state = 'indexed'
    ),
    total_symbols = (
        SELECT COALESCE(SUM(symbol_count), 0) 
        FROM file_metadata 
        WHERE index_id = NEW.index_id AND processing_state = 'indexed'
    ),
    updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.index_id;
END;

CREATE TRIGGER update_session_activity_on_query
AFTER UPDATE OF query_count ON mcp_query_sessions
BEGIN
    UPDATE mcp_query_sessions 
    SET last_activity = CURRENT_TIMESTAMP 
    WHERE session_id = NEW.session_id;
END;
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::path::Path;

    fn create_test_db() -> Result<Connection> {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        Connection::open(db_path)
    }

    #[test]
    fn test_schema_migration() {
        let conn = create_test_db().unwrap();
        let mut migrator = SchemaMigrator::new(conn);
        
        // Initial version should be 0
        assert_eq!(migrator.get_current_version().unwrap(), 0);
        
        // Run migrations
        migrator.migrate().unwrap();
        
        // Version should now be current
        assert_eq!(migrator.get_current_version().unwrap(), CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_tables_created() {
        let conn = create_test_db().unwrap();
        let mut migrator = SchemaMigrator::new(conn);
        migrator.migrate().unwrap();
        
        let conn = migrator.into_connection();
        
        // Check that all tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        
        let expected_tables = vec![
            "code_elements",
            "code_indices", 
            "file_metadata",
            "mcp_query_sessions",
            "schema_migrations",
            "symbol_relationships",
        ];
        
        for expected_table in expected_tables {
            assert!(tables.contains(&expected_table.to_string()));
        }
        
        Ok(())
    }

    #[test]
    fn test_indices_created() {
        let conn = create_test_db().unwrap();
        let mut migrator = SchemaMigrator::new(conn);
        migrator.migrate().unwrap();
        
        let conn = migrator.into_connection();
        
        // Check that indices exist
        let indices: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%' ORDER BY name")?
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        
        // Should have created multiple indices
        assert!(indices.len() > 10);
        assert!(indices.iter().any(|name| name.contains("code_elements")));
        assert!(indices.iter().any(|name| name.contains("file_metadata")));
        assert!(indices.iter().any(|name| name.contains("symbol_relationships")));
        
        Ok(())
    }

    #[test]
    fn test_views_created() {
        let conn = create_test_db().unwrap();
        let mut migrator = SchemaMigrator::new(conn);
        migrator.migrate().unwrap();
        
        let conn = migrator.into_connection();
        
        // Check that views exist
        let views: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='view' ORDER BY name")?
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        
        assert!(views.contains(&"symbol_details_view".to_string()));
        assert!(views.contains(&"file_stats_view".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let conn = create_test_db().unwrap();
        let mut migrator = SchemaMigrator::new(conn);
        migrator.migrate().unwrap();
        
        let conn = migrator.into_connection();
        
        // Check foreign keys are enabled
        let foreign_keys: i32 = conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
        assert_eq!(foreign_keys, 1);
        
        Ok(())
    }

    #[test]
    fn test_migration_idempotent() {
        let conn = create_test_db().unwrap();
        let mut migrator = SchemaMigrator::new(conn);
        
        // Run migrations twice
        migrator.migrate().unwrap();
        migrator.migrate().unwrap();
        
        // Version should still be current
        assert_eq!(migrator.get_current_version().unwrap(), CURRENT_SCHEMA_VERSION);
    }
}