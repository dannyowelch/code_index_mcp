use rusqlite::{Connection, Result, params, Row};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

use crate::lib::storage::models::code_index::{CodeIndex, IndexState};
use crate::lib::storage::models::code_element::{CodeElement, SymbolType, AccessModifier};
use crate::lib::storage::models::file_metadata::{FileMetadata, FileProcessingState};
use crate::lib::storage::models::symbol_relationships::{SymbolRelationship, RelationshipType, RelationshipQuery};
use crate::lib::storage::models::mcp_query_session::{McpQuerySession, SessionStatus, SessionQuery};

/// Repository providing CRUD operations for all storage models
pub struct Repository {
    connection: Connection,
}

impl Repository {
    /// Creates a new repository with the given database connection
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    /// Returns a reference to the underlying connection
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Consumes the repository and returns the connection
    pub fn into_connection(self) -> Connection {
        self.connection
    }

    // === Code Index CRUD Operations ===

    /// Creates a new code index
    pub fn create_code_index(&self, mut index: CodeIndex) -> Result<CodeIndex> {
        index.validate().map_err(|e| rusqlite::Error::InvalidColumnName(e))?;
        
        self.connection.execute(
            r#"
            INSERT INTO code_indices (
                id, name, base_path, created_at, updated_at, 
                total_files, total_symbols, index_version, state
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                index.id.to_string(),
                index.name,
                index.base_path,
                index.created_at.to_rfc3339(),
                index.updated_at.to_rfc3339(),
                index.total_files,
                index.total_symbols,
                index.index_version,
                "creating"
            ],
        )?;
        
        Ok(index)
    }

    /// Retrieves a code index by ID
    pub fn get_code_index(&self, id: &Uuid) -> Result<Option<CodeIndex>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, base_path, created_at, updated_at, total_files, total_symbols, index_version, state FROM code_indices WHERE id = ?1"
        )?;
        
        let mut rows = stmt.query_map([id.to_string()], |row| {
            Ok(self.row_to_code_index(row)?)
        })?;
        
        match rows.next() {
            Some(index) => Ok(Some(index?)),
            None => Ok(None),
        }
    }

    /// Retrieves a code index by name
    pub fn get_code_index_by_name(&self, name: &str) -> Result<Option<CodeIndex>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, base_path, created_at, updated_at, total_files, total_symbols, index_version, state FROM code_indices WHERE name = ?1"
        )?;
        
        let mut rows = stmt.query_map([name], |row| {
            Ok(self.row_to_code_index(row)?)
        })?;
        
        match rows.next() {
            Some(index) => Ok(Some(index?)),
            None => Ok(None),
        }
    }

    /// Lists all code indices
    pub fn list_code_indices(&self) -> Result<Vec<CodeIndex>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, base_path, created_at, updated_at, total_files, total_symbols, index_version, state FROM code_indices ORDER BY name"
        )?;
        
        let indices = stmt.query_map([], |row| {
            Ok(self.row_to_code_index(row)?)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(indices)
    }

    /// Updates a code index
    pub fn update_code_index(&self, index: &CodeIndex) -> Result<()> {
        index.validate().map_err(|e| rusqlite::Error::InvalidColumnName(e))?;
        
        let rows_affected = self.connection.execute(
            r#"
            UPDATE code_indices SET 
                name = ?2, base_path = ?3, updated_at = ?4,
                total_files = ?5, total_symbols = ?6, index_version = ?7
            WHERE id = ?1
            "#,
            params![
                index.id.to_string(),
                index.name,
                index.base_path,
                index.updated_at.to_rfc3339(),
                index.total_files,
                index.total_symbols,
                index.index_version
            ],
        )?;
        
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        
        Ok(())
    }

    /// Updates the state of a code index
    pub fn update_code_index_state(&self, id: &Uuid, state: IndexState) -> Result<()> {
        let state_str = match state {
            IndexState::Creating => "creating",
            IndexState::Active => "active",
            IndexState::Updating => "updating",
            IndexState::Archived => "archived",
            IndexState::Failed => "failed",
        };
        
        let rows_affected = self.connection.execute(
            "UPDATE code_indices SET state = ?2, updated_at = ?3 WHERE id = ?1",
            params![id.to_string(), state_str, Utc::now().to_rfc3339()],
        )?;
        
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        
        Ok(())
    }

    /// Deletes a code index and all related data
    pub fn delete_code_index(&self, id: &Uuid) -> Result<()> {
        let rows_affected = self.connection.execute(
            "DELETE FROM code_indices WHERE id = ?1",
            [id.to_string()],
        )?;
        
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        
        Ok(())
    }

    // === File Metadata CRUD Operations ===

    /// Creates a new file metadata entry
    pub fn create_file_metadata(&self, mut metadata: FileMetadata) -> Result<FileMetadata> {
        metadata.validate().map_err(|e| rusqlite::Error::InvalidColumnName(e))?;
        
        self.connection.execute(
            r#"
            INSERT INTO file_metadata (
                index_id, file_path, file_hash, last_modified, 
                size_bytes, symbol_count, indexed_at, processing_state
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                metadata.index_id.to_string(),
                metadata.file_path,
                metadata.file_hash,
                metadata.last_modified.to_rfc3339(),
                metadata.size_bytes,
                metadata.symbol_count,
                metadata.indexed_at.to_rfc3339(),
                "pending"
            ],
        )?;
        
        metadata.id = Some(self.connection.last_insert_rowid());
        Ok(metadata)
    }

    /// Retrieves file metadata by ID
    pub fn get_file_metadata(&self, id: i64) -> Result<Option<FileMetadata>> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT id, index_id, file_path, file_hash, last_modified, 
                   size_bytes, symbol_count, indexed_at, processing_state 
            FROM file_metadata WHERE id = ?1
            "#
        )?;
        
        let mut rows = stmt.query_map([id], |row| {
            Ok(self.row_to_file_metadata(row)?)
        })?;
        
        match rows.next() {
            Some(metadata) => Ok(Some(metadata?)),
            None => Ok(None),
        }
    }

    /// Retrieves file metadata by index and file path
    pub fn get_file_metadata_by_path(&self, index_id: &Uuid, file_path: &str) -> Result<Option<FileMetadata>> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT id, index_id, file_path, file_hash, last_modified, 
                   size_bytes, symbol_count, indexed_at, processing_state 
            FROM file_metadata WHERE index_id = ?1 AND file_path = ?2
            "#
        )?;
        
        let mut rows = stmt.query_map(params![index_id.to_string(), file_path], |row| {
            Ok(self.row_to_file_metadata(row)?)
        })?;
        
        match rows.next() {
            Some(metadata) => Ok(Some(metadata?)),
            None => Ok(None),
        }
    }

    /// Lists file metadata for an index
    pub fn list_file_metadata(&self, index_id: &Uuid) -> Result<Vec<FileMetadata>> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT id, index_id, file_path, file_hash, last_modified, 
                   size_bytes, symbol_count, indexed_at, processing_state 
            FROM file_metadata WHERE index_id = ?1 ORDER BY file_path
            "#
        )?;
        
        let metadata_list = stmt.query_map([index_id.to_string()], |row| {
            Ok(self.row_to_file_metadata(row)?)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(metadata_list)
    }

    /// Updates file metadata
    pub fn update_file_metadata(&self, metadata: &FileMetadata) -> Result<()> {
        metadata.validate().map_err(|e| rusqlite::Error::InvalidColumnName(e))?;
        
        let id = metadata.id.ok_or(rusqlite::Error::InvalidColumnName("File metadata ID is required".to_string()))?;
        
        let rows_affected = self.connection.execute(
            r#"
            UPDATE file_metadata SET 
                file_hash = ?2, last_modified = ?3, size_bytes = ?4,
                symbol_count = ?5, indexed_at = ?6, processing_state = ?7
            WHERE id = ?1
            "#,
            params![
                id,
                metadata.file_hash,
                metadata.last_modified.to_rfc3339(),
                metadata.size_bytes,
                metadata.symbol_count,
                metadata.indexed_at.to_rfc3339(),
                "indexed"
            ],
        )?;
        
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        
        Ok(())
    }

    /// Updates file processing state
    pub fn update_file_processing_state(&self, id: i64, state: FileProcessingState) -> Result<()> {
        let state_str = match state {
            FileProcessingState::Pending => "pending",
            FileProcessingState::Processing => "processing", 
            FileProcessingState::Indexed => "indexed",
            FileProcessingState::Error => "error",
        };
        
        let rows_affected = self.connection.execute(
            "UPDATE file_metadata SET processing_state = ?2 WHERE id = ?1",
            params![id, state_str],
        )?;
        
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        
        Ok(())
    }

    /// Deletes file metadata
    pub fn delete_file_metadata(&self, id: i64) -> Result<()> {
        let rows_affected = self.connection.execute(
            "DELETE FROM file_metadata WHERE id = ?1",
            [id],
        )?;
        
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        
        Ok(())
    }

    // === Code Element CRUD Operations ===

    /// Creates a new code element
    pub fn create_code_element(&self, mut element: CodeElement) -> Result<CodeElement> {
        element.validate().map_err(|e| rusqlite::Error::InvalidColumnName(e))?;
        
        self.connection.execute(
            r#"
            INSERT INTO code_elements (
                index_id, symbol_name, symbol_type, file_path, line_number,
                column_number, definition_hash, scope, access_modifier, 
                is_declaration, signature
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                element.index_id.to_string(),
                element.symbol_name,
                element.symbol_type.as_str(),
                element.file_path,
                element.line_number,
                element.column_number,
                element.definition_hash,
                element.scope,
                element.access_modifier.map(|a| a.as_str()),
                element.is_declaration,
                element.signature
            ],
        )?;
        
        element.id = Some(self.connection.last_insert_rowid());
        Ok(element)
    }

    /// Retrieves a code element by ID
    pub fn get_code_element(&self, id: i64) -> Result<Option<CodeElement>> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT id, index_id, symbol_name, symbol_type, file_path, line_number,
                   column_number, definition_hash, scope, access_modifier, 
                   is_declaration, signature
            FROM code_elements WHERE id = ?1
            "#
        )?;
        
        let mut rows = stmt.query_map([id], |row| {
            Ok(self.row_to_code_element(row)?)
        })?;
        
        match rows.next() {
            Some(element) => Ok(Some(element?)),
            None => Ok(None),
        }
    }

    /// Searches for code elements by symbol name pattern
    pub fn search_code_elements(&self, index_id: &Uuid, name_pattern: &str, symbol_types: Option<&[SymbolType]>) -> Result<Vec<CodeElement>> {
        let mut query = String::from(
            r#"
            SELECT id, index_id, symbol_name, symbol_type, file_path, line_number,
                   column_number, definition_hash, scope, access_modifier, 
                   is_declaration, signature
            FROM code_elements 
            WHERE index_id = ?1 AND symbol_name LIKE ?2
            "#
        );
        
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![
            Box::new(index_id.to_string()),
            Box::new(format!("%{}%", name_pattern)),
        ];
        
        if let Some(types) = symbol_types {
            if !types.is_empty() {
                query.push_str(" AND symbol_type IN (");
                for (i, symbol_type) in types.iter().enumerate() {
                    if i > 0 {
                        query.push_str(", ");
                    }
                    query.push_str(&format!("?{}", params.len() + 1));
                    params.push(Box::new(symbol_type.as_str().to_string()));
                }
                query.push(')');
            }
        }
        
        query.push_str(" ORDER BY symbol_name, file_path");
        
        let mut stmt = self.connection.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let elements = stmt.query_map(&param_refs[..], |row| {
            Ok(self.row_to_code_element(row)?)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(elements)
    }

    /// Lists code elements for a file
    pub fn list_code_elements_by_file(&self, index_id: &Uuid, file_path: &str) -> Result<Vec<CodeElement>> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT id, index_id, symbol_name, symbol_type, file_path, line_number,
                   column_number, definition_hash, scope, access_modifier, 
                   is_declaration, signature
            FROM code_elements 
            WHERE index_id = ?1 AND file_path = ?2 
            ORDER BY line_number, column_number
            "#
        )?;
        
        let elements = stmt.query_map(params![index_id.to_string(), file_path], |row| {
            Ok(self.row_to_code_element(row)?)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(elements)
    }

    /// Updates a code element
    pub fn update_code_element(&self, element: &CodeElement) -> Result<()> {
        element.validate().map_err(|e| rusqlite::Error::InvalidColumnName(e))?;
        
        let id = element.id.ok_or(rusqlite::Error::InvalidColumnName("Code element ID is required".to_string()))?;
        
        let rows_affected = self.connection.execute(
            r#"
            UPDATE code_elements SET 
                symbol_name = ?2, symbol_type = ?3, file_path = ?4, line_number = ?5,
                column_number = ?6, definition_hash = ?7, scope = ?8, 
                access_modifier = ?9, is_declaration = ?10, signature = ?11
            WHERE id = ?1
            "#,
            params![
                id,
                element.symbol_name,
                element.symbol_type.as_str(),
                element.file_path,
                element.line_number,
                element.column_number,
                element.definition_hash,
                element.scope,
                element.access_modifier.map(|a| a.as_str()),
                element.is_declaration,
                element.signature
            ],
        )?;
        
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        
        Ok(())
    }

    /// Deletes code elements for a file (used during re-indexing)
    pub fn delete_code_elements_by_file(&self, index_id: &Uuid, file_path: &str) -> Result<()> {
        self.connection.execute(
            "DELETE FROM code_elements WHERE index_id = ?1 AND file_path = ?2",
            params![index_id.to_string(), file_path],
        )?;
        
        Ok(())
    }

    /// Deletes a code element by ID
    pub fn delete_code_element(&self, id: i64) -> Result<()> {
        let rows_affected = self.connection.execute(
            "DELETE FROM code_elements WHERE id = ?1",
            [id],
        )?;
        
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        
        Ok(())
    }

    // === Symbol Relationship CRUD Operations ===

    /// Creates a new symbol relationship
    pub fn create_symbol_relationship(&self, mut relationship: SymbolRelationship) -> Result<SymbolRelationship> {
        relationship.validate().map_err(|e| rusqlite::Error::InvalidColumnName(e))?;
        
        self.connection.execute(
            r#"
            INSERT INTO symbol_relationships (
                from_symbol_id, to_symbol_id, relationship_type, 
                file_path, line_number
            ) VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                relationship.from_symbol_id,
                relationship.to_symbol_id,
                relationship.relationship_type.as_str(),
                relationship.file_path,
                relationship.line_number
            ],
        )?;
        
        relationship.id = Some(self.connection.last_insert_rowid());
        Ok(relationship)
    }

    /// Queries symbol relationships using the relationship query builder
    pub fn query_symbol_relationships(&self, query: &RelationshipQuery) -> Result<Vec<SymbolRelationship>> {
        let mut sql = String::from(
            r#"
            SELECT id, from_symbol_id, to_symbol_id, relationship_type, file_path, line_number
            FROM symbol_relationships WHERE 1=1
            "#
        );
        
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
        
        if let Some(from_id) = query.from_symbol_id {
            sql.push_str(&format!(" AND from_symbol_id = ?{}", params.len() + 1));
            params.push(Box::new(from_id));
        }
        
        if let Some(to_id) = query.to_symbol_id {
            sql.push_str(&format!(" AND to_symbol_id = ?{}", params.len() + 1));
            params.push(Box::new(to_id));
        }
        
        if !query.relationship_types.is_empty() {
            sql.push_str(" AND relationship_type IN (");
            for (i, rel_type) in query.relationship_types.iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ");
                }
                sql.push_str(&format!("?{}", params.len() + 1));
                params.push(Box::new(rel_type.as_str().to_string()));
            }
            sql.push(')');
        }
        
        if let Some(pattern) = &query.file_path_pattern {
            sql.push_str(&format!(" AND file_path LIKE ?{}", params.len() + 1));
            params.push(Box::new(format!("%{}%", pattern)));
        }
        
        sql.push_str(" ORDER BY from_symbol_id, to_symbol_id");
        
        let mut stmt = self.connection.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let relationships = stmt.query_map(&param_refs[..], |row| {
            Ok(self.row_to_symbol_relationship(row)?)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(relationships)
    }

    /// Lists all relationships for a symbol (both incoming and outgoing)
    pub fn get_symbol_relationships(&self, symbol_id: i64) -> Result<(Vec<SymbolRelationship>, Vec<SymbolRelationship>)> {
        // Outgoing relationships (from this symbol)
        let outgoing = self.query_symbol_relationships(
            &RelationshipQuery::new().from_symbol(symbol_id)
        )?;
        
        // Incoming relationships (to this symbol)
        let incoming = self.query_symbol_relationships(
            &RelationshipQuery::new().to_symbol(symbol_id)
        )?;
        
        Ok((outgoing, incoming))
    }

    /// Deletes symbol relationships for a file (used during re-indexing)
    pub fn delete_symbol_relationships_by_file(&self, file_path: &str) -> Result<()> {
        self.connection.execute(
            "DELETE FROM symbol_relationships WHERE file_path = ?1",
            [file_path],
        )?;
        
        Ok(())
    }

    /// Deletes a symbol relationship by ID
    pub fn delete_symbol_relationship(&self, id: i64) -> Result<()> {
        let rows_affected = self.connection.execute(
            "DELETE FROM symbol_relationships WHERE id = ?1",
            [id],
        )?;
        
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        
        Ok(())
    }

    // === MCP Query Session CRUD Operations ===

    /// Creates a new MCP query session
    pub fn create_mcp_session(&self, mut session: McpQuerySession) -> Result<McpQuerySession> {
        session.validate().map_err(|e| rusqlite::Error::InvalidColumnName(e))?;
        
        self.connection.execute(
            r#"
            INSERT INTO mcp_query_sessions (
                session_id, client_name, active_index_id, created_at, 
                last_activity, query_count, status, client_metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                session.session_id.to_string(),
                session.client_name,
                session.active_index_id.map(|id| id.to_string()),
                session.created_at.to_rfc3339(),
                session.last_activity.to_rfc3339(),
                session.query_count,
                session.status.as_str(),
                session.client_metadata
            ],
        )?;
        
        Ok(session)
    }

    /// Retrieves an MCP session by ID
    pub fn get_mcp_session(&self, session_id: &Uuid) -> Result<Option<McpQuerySession>> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT session_id, client_name, active_index_id, created_at, 
                   last_activity, query_count, status, client_metadata
            FROM mcp_query_sessions WHERE session_id = ?1
            "#
        )?;
        
        let mut rows = stmt.query_map([session_id.to_string()], |row| {
            Ok(self.row_to_mcp_session(row)?)
        })?;
        
        match rows.next() {
            Some(session) => Ok(Some(session?)),
            None => Ok(None),
        }
    }

    /// Queries MCP sessions using the session query builder
    pub fn query_mcp_sessions(&self, query: &SessionQuery) -> Result<Vec<McpQuerySession>> {
        let mut sql = String::from(
            r#"
            SELECT session_id, client_name, active_index_id, created_at, 
                   last_activity, query_count, status, client_metadata
            FROM mcp_query_sessions WHERE 1=1
            "#
        );
        
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
        
        if let Some(pattern) = &query.client_name_pattern {
            sql.push_str(&format!(" AND client_name LIKE ?{}", params.len() + 1));
            params.push(Box::new(format!("%{}%", pattern)));
        }
        
        if let Some(status) = &query.status_filter {
            sql.push_str(&format!(" AND status = ?{}", params.len() + 1));
            params.push(Box::new(status.as_str().to_string()));
        }
        
        if let Some(index_id) = &query.active_index_id {
            sql.push_str(&format!(" AND active_index_id = ?{}", params.len() + 1));
            params.push(Box::new(index_id.to_string()));
        }
        
        if let Some(created_after) = &query.created_after {
            sql.push_str(&format!(" AND created_at >= ?{}", params.len() + 1));
            params.push(Box::new(created_after.to_rfc3339()));
        }
        
        if let Some(created_before) = &query.created_before {
            sql.push_str(&format!(" AND created_at <= ?{}", params.len() + 1));
            params.push(Box::new(created_before.to_rfc3339()));
        }
        
        if let Some(min_queries) = &query.min_queries {
            sql.push_str(&format!(" AND query_count >= ?{}", params.len() + 1));
            params.push(Box::new(*min_queries));
        }
        
        if let Some(idle_duration) = &query.idle_longer_than {
            let cutoff_time = Utc::now() - *idle_duration;
            sql.push_str(&format!(" AND last_activity <= ?{}", params.len() + 1));
            params.push(Box::new(cutoff_time.to_rfc3339()));
        }
        
        sql.push_str(" ORDER BY last_activity DESC");
        
        let mut stmt = self.connection.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let sessions = stmt.query_map(&param_refs[..], |row| {
            Ok(self.row_to_mcp_session(row)?)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(sessions)
    }

    /// Updates an MCP session
    pub fn update_mcp_session(&self, session: &McpQuerySession) -> Result<()> {
        session.validate().map_err(|e| rusqlite::Error::InvalidColumnName(e))?;
        
        let rows_affected = self.connection.execute(
            r#"
            UPDATE mcp_query_sessions SET 
                client_name = ?2, active_index_id = ?3, last_activity = ?4,
                query_count = ?5, status = ?6, client_metadata = ?7
            WHERE session_id = ?1
            "#,
            params![
                session.session_id.to_string(),
                session.client_name,
                session.active_index_id.map(|id| id.to_string()),
                session.last_activity.to_rfc3339(),
                session.query_count,
                session.status.as_str(),
                session.client_metadata
            ],
        )?;
        
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        
        Ok(())
    }

    /// Deletes an MCP session
    pub fn delete_mcp_session(&self, session_id: &Uuid) -> Result<()> {
        let rows_affected = self.connection.execute(
            "DELETE FROM mcp_query_sessions WHERE session_id = ?1",
            [session_id.to_string()],
        )?;
        
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        
        Ok(())
    }

    // === Utility Methods ===

    /// Gets statistics for all indices
    pub fn get_index_statistics(&self) -> Result<HashMap<String, IndexStatistics>> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT 
                ci.id, ci.name, ci.total_files, ci.total_symbols,
                COUNT(DISTINCT fm.id) as file_count,
                COUNT(DISTINCT ce.id) as element_count,
                COUNT(DISTINCT sr.id) as relationship_count
            FROM code_indices ci
            LEFT JOIN file_metadata fm ON ci.id = fm.index_id AND fm.processing_state = 'indexed'
            LEFT JOIN code_elements ce ON ci.id = ce.index_id  
            LEFT JOIN symbol_relationships sr ON ce.id = sr.from_symbol_id
            GROUP BY ci.id, ci.name, ci.total_files, ci.total_symbols
            "#
        )?;
        
        let mut stats_map = HashMap::new();
        
        let rows = stmt.query_map([], |row| {
            let index_id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let total_files: u32 = row.get(2)?;
            let total_symbols: u32 = row.get(3)?;
            let actual_file_count: i64 = row.get(4)?;
            let actual_element_count: i64 = row.get(5)?;
            let relationship_count: i64 = row.get(6)?;
            
            Ok((name.clone(), IndexStatistics {
                index_id: Uuid::parse_str(&index_id).unwrap(),
                name,
                reported_files: total_files,
                reported_symbols: total_symbols,
                actual_files: actual_file_count as u32,
                actual_elements: actual_element_count as u32,
                relationships: relationship_count as u32,
            }))
        })?;
        
        for row in rows {
            let (name, stats) = row?;
            stats_map.insert(name, stats);
        }
        
        Ok(stats_map)
    }

    // === Private Helper Methods ===

    fn row_to_code_index(&self, row: &Row) -> Result<CodeIndex> {
        let id_str: String = row.get(0)?;
        let created_at_str: String = row.get(3)?;
        let updated_at_str: String = row.get(4)?;
        let state_str: String = row.get(8)?;
        
        Ok(CodeIndex {
            id: Uuid::parse_str(&id_str).map_err(|_| rusqlite::Error::InvalidColumnType(0, "Invalid UUID".to_string(), rusqlite::types::Type::Text))?,
            name: row.get(1)?,
            base_path: row.get(2)?,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(3, "Invalid datetime".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "Invalid datetime".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            total_files: row.get(5)?,
            total_symbols: row.get(6)?,
            index_version: row.get(7)?,
        })
    }

    fn row_to_file_metadata(&self, row: &Row) -> Result<FileMetadata> {
        let index_id_str: String = row.get(1)?;
        let last_modified_str: String = row.get(4)?;
        let indexed_at_str: String = row.get(7)?;
        
        Ok(FileMetadata {
            id: Some(row.get(0)?),
            index_id: Uuid::parse_str(&index_id_str).map_err(|_| rusqlite::Error::InvalidColumnType(1, "Invalid UUID".to_string(), rusqlite::types::Type::Text))?,
            file_path: row.get(2)?,
            file_hash: row.get(3)?,
            last_modified: DateTime::parse_from_rfc3339(&last_modified_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "Invalid datetime".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            size_bytes: row.get(5)?,
            symbol_count: row.get(6)?,
            indexed_at: DateTime::parse_from_rfc3339(&indexed_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(7, "Invalid datetime".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }

    fn row_to_code_element(&self, row: &Row) -> Result<CodeElement> {
        let index_id_str: String = row.get(1)?;
        let symbol_type_str: String = row.get(3)?;
        let access_modifier_str: Option<String> = row.get(9)?;
        
        let symbol_type = match symbol_type_str.as_str() {
            "function" => SymbolType::Function,
            "class" => SymbolType::Class,
            "struct" => SymbolType::Struct,
            "variable" => SymbolType::Variable,
            "macro" => SymbolType::Macro,
            "namespace" => SymbolType::Namespace,
            "enum" => SymbolType::Enum,
            "typedef" => SymbolType::Typedef,
            "union" => SymbolType::Union,
            "template" => SymbolType::Template,
            "constructor" => SymbolType::Constructor,
            "destructor" => SymbolType::Destructor,
            "operator" => SymbolType::Operator,
            _ => return Err(rusqlite::Error::InvalidColumnType(3, "Invalid symbol type".to_string(), rusqlite::types::Type::Text)),
        };
        
        let access_modifier = access_modifier_str.as_ref().map(|s| match s.as_str() {
            "public" => Ok(AccessModifier::Public),
            "private" => Ok(AccessModifier::Private),
            "protected" => Ok(AccessModifier::Protected),
            _ => Err(rusqlite::Error::InvalidColumnType(9, "Invalid access modifier".to_string(), rusqlite::types::Type::Text)),
        }).transpose()?;
        
        Ok(CodeElement {
            id: Some(row.get(0)?),
            index_id: Uuid::parse_str(&index_id_str).map_err(|_| rusqlite::Error::InvalidColumnType(1, "Invalid UUID".to_string(), rusqlite::types::Type::Text))?,
            symbol_name: row.get(2)?,
            symbol_type,
            file_path: row.get(4)?,
            line_number: row.get(5)?,
            column_number: row.get(6)?,
            definition_hash: row.get(7)?,
            scope: row.get(8)?,
            access_modifier,
            is_declaration: row.get(10)?,
            signature: row.get(11)?,
        })
    }

    fn row_to_symbol_relationship(&self, row: &Row) -> Result<SymbolRelationship> {
        let relationship_type_str: String = row.get(3)?;
        
        let relationship_type = match relationship_type_str.as_str() {
            "inherits" => RelationshipType::Inherits,
            "uses" => RelationshipType::Uses,
            "includes" => RelationshipType::Includes,
            "calls" => RelationshipType::Calls,
            "defines" => RelationshipType::Defines,
            "instantiates" => RelationshipType::Instantiates,
            "contained_in" => RelationshipType::ContainedIn,
            "friend" => RelationshipType::Friend,
            "overrides" => RelationshipType::Overrides,
            "specializes" => RelationshipType::Specializes,
            _ => return Err(rusqlite::Error::InvalidColumnType(3, "Invalid relationship type".to_string(), rusqlite::types::Type::Text)),
        };
        
        Ok(SymbolRelationship {
            id: Some(row.get(0)?),
            from_symbol_id: row.get(1)?,
            to_symbol_id: row.get(2)?,
            relationship_type,
            file_path: row.get(4)?,
            line_number: row.get(5)?,
        })
    }

    fn row_to_mcp_session(&self, row: &Row) -> Result<McpQuerySession> {
        let session_id_str: String = row.get(0)?;
        let active_index_id_str: Option<String> = row.get(2)?;
        let created_at_str: String = row.get(3)?;
        let last_activity_str: String = row.get(4)?;
        let status_str: String = row.get(6)?;
        
        let status = match status_str.as_str() {
            "active" => SessionStatus::Active,
            "inactive" => SessionStatus::Inactive,
            "terminated" => SessionStatus::Terminated,
            "error" => SessionStatus::Error,
            _ => return Err(rusqlite::Error::InvalidColumnType(6, "Invalid session status".to_string(), rusqlite::types::Type::Text)),
        };
        
        let active_index_id = active_index_id_str
            .map(|s| Uuid::parse_str(&s))
            .transpose()
            .map_err(|_| rusqlite::Error::InvalidColumnType(2, "Invalid UUID".to_string(), rusqlite::types::Type::Text))?;
        
        Ok(McpQuerySession {
            session_id: Uuid::parse_str(&session_id_str).map_err(|_| rusqlite::Error::InvalidColumnType(0, "Invalid UUID".to_string(), rusqlite::types::Type::Text))?,
            client_name: row.get(1)?,
            active_index_id,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(3, "Invalid datetime".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            last_activity: DateTime::parse_from_rfc3339(&last_activity_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "Invalid datetime".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            query_count: row.get(5)?,
            status,
            client_metadata: row.get(7)?,
        })
    }
}

/// Statistics for a code index
#[derive(Debug, Clone)]
pub struct IndexStatistics {
    pub index_id: Uuid,
    pub name: String,
    pub reported_files: u32,
    pub reported_symbols: u32,
    pub actual_files: u32,
    pub actual_elements: u32,
    pub relationships: u32,
}

impl IndexStatistics {
    /// Returns true if the reported counts match actual counts
    pub fn is_consistent(&self) -> bool {
        self.reported_files == self.actual_files && self.reported_symbols == self.actual_elements
    }
    
    /// Returns the difference between reported and actual file counts
    pub fn file_count_difference(&self) -> i32 {
        self.actual_files as i32 - self.reported_files as i32
    }
    
    /// Returns the difference between reported and actual symbol counts
    pub fn symbol_count_difference(&self) -> i32 {
        self.actual_elements as i32 - self.reported_symbols as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::storage::connection::{DatabaseConfig, DatabaseManager};
    use chrono::TimeZone;

    fn create_test_repository() -> Repository {
        let config = DatabaseConfig::in_memory();
        let manager = DatabaseManager::new(config).unwrap();
        let connection = manager.connect().unwrap();
        Repository::new(connection)
    }

    #[test]
    fn test_code_index_crud() {
        let repo = create_test_repository();
        let index = CodeIndex::new("Test Index".to_string(), "/test/path".to_string());
        let index_id = index.id;
        
        // Create
        let created_index = repo.create_code_index(index).unwrap();
        assert_eq!(created_index.name, "Test Index");
        
        // Read by ID
        let retrieved_index = repo.get_code_index(&index_id).unwrap().unwrap();
        assert_eq!(retrieved_index.name, "Test Index");
        assert_eq!(retrieved_index.base_path, "/test/path");
        
        // Read by name
        let retrieved_by_name = repo.get_code_index_by_name("Test Index").unwrap().unwrap();
        assert_eq!(retrieved_by_name.id, index_id);
        
        // Update
        let mut updated_index = retrieved_index;
        updated_index.name = "Updated Test Index".to_string();
        repo.update_code_index(&updated_index).unwrap();
        
        let retrieved_updated = repo.get_code_index(&index_id).unwrap().unwrap();
        assert_eq!(retrieved_updated.name, "Updated Test Index");
        
        // List
        let indices = repo.list_code_indices().unwrap();
        assert_eq!(indices.len(), 1);
        assert_eq!(indices[0].name, "Updated Test Index");
        
        // Delete
        repo.delete_code_index(&index_id).unwrap();
        assert!(repo.get_code_index(&index_id).unwrap().is_none());
    }

    #[test]
    fn test_file_metadata_crud() {
        let repo = create_test_repository();
        
        // Create an index first
        let index = CodeIndex::new("Test Index".to_string(), "/test/path".to_string());
        let index_id = index.id;
        repo.create_code_index(index).unwrap();
        
        // Create file metadata
        let metadata = FileMetadata::new(
            index_id,
            "src/test.cpp".to_string(),
            "a".repeat(64),
            Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
            1024,
        );
        
        let created_metadata = repo.create_file_metadata(metadata).unwrap();
        assert!(created_metadata.id.is_some());
        
        let metadata_id = created_metadata.id.unwrap();
        
        // Read by ID
        let retrieved_metadata = repo.get_file_metadata(metadata_id).unwrap().unwrap();
        assert_eq!(retrieved_metadata.file_path, "src/test.cpp");
        
        // Read by path
        let retrieved_by_path = repo.get_file_metadata_by_path(&index_id, "src/test.cpp").unwrap().unwrap();
        assert_eq!(retrieved_by_path.id, Some(metadata_id));
        
        // List
        let metadata_list = repo.list_file_metadata(&index_id).unwrap();
        assert_eq!(metadata_list.len(), 1);
        
        // Update
        let mut updated_metadata = retrieved_metadata;
        updated_metadata.symbol_count = 42;
        repo.update_file_metadata(&updated_metadata).unwrap();
        
        let retrieved_updated = repo.get_file_metadata(metadata_id).unwrap().unwrap();
        assert_eq!(retrieved_updated.symbol_count, 42);
        
        // Delete
        repo.delete_file_metadata(metadata_id).unwrap();
        assert!(repo.get_file_metadata(metadata_id).unwrap().is_none());
    }

    #[test]
    fn test_code_element_crud() {
        let repo = create_test_repository();
        
        // Create an index first
        let index = CodeIndex::new("Test Index".to_string(), "/test/path".to_string());
        let index_id = index.id;
        repo.create_code_index(index).unwrap();
        
        // Create code element
        let element = CodeElement::new(
            index_id,
            "testFunction".to_string(),
            SymbolType::Function,
            "src/test.cpp".to_string(),
            10,
            5,
            "a".repeat(64),
        );
        
        let created_element = repo.create_code_element(element).unwrap();
        assert!(created_element.id.is_some());
        
        let element_id = created_element.id.unwrap();
        
        // Read by ID
        let retrieved_element = repo.get_code_element(element_id).unwrap().unwrap();
        assert_eq!(retrieved_element.symbol_name, "testFunction");
        
        // Search by name
        let search_results = repo.search_code_elements(&index_id, "test", None).unwrap();
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].symbol_name, "testFunction");
        
        // List by file
        let file_elements = repo.list_code_elements_by_file(&index_id, "src/test.cpp").unwrap();
        assert_eq!(file_elements.len(), 1);
        
        // Update
        let mut updated_element = retrieved_element;
        updated_element.symbol_name = "updatedFunction".to_string();
        repo.update_code_element(&updated_element).unwrap();
        
        let retrieved_updated = repo.get_code_element(element_id).unwrap().unwrap();
        assert_eq!(retrieved_updated.symbol_name, "updatedFunction");
        
        // Delete
        repo.delete_code_element(element_id).unwrap();
        assert!(repo.get_code_element(element_id).unwrap().is_none());
    }

    #[test]
    fn test_symbol_relationship_crud() {
        let repo = create_test_repository();
        
        // Create an index and elements first
        let index = CodeIndex::new("Test Index".to_string(), "/test/path".to_string());
        let index_id = index.id;
        repo.create_code_index(index).unwrap();
        
        let element1 = repo.create_code_element(CodeElement::new(
            index_id,
            "ClassA".to_string(),
            SymbolType::Class,
            "src/test.h".to_string(),
            10,
            1,
            "a".repeat(64),
        )).unwrap();
        
        let element2 = repo.create_code_element(CodeElement::new(
            index_id,
            "ClassB".to_string(),
            SymbolType::Class,
            "src/test.h".to_string(),
            20,
            1,
            "b".repeat(64),
        )).unwrap();
        
        let element1_id = element1.id.unwrap();
        let element2_id = element2.id.unwrap();
        
        // Create relationship
        let relationship = SymbolRelationship::new(
            element2_id,
            element1_id,
            RelationshipType::Inherits,
            "src/test.h".to_string(),
            20,
        );
        
        let created_relationship = repo.create_symbol_relationship(relationship).unwrap();
        assert!(created_relationship.id.is_some());
        
        // Query relationships
        let query = RelationshipQuery::new().from_symbol(element2_id);
        let relationships = repo.query_symbol_relationships(&query).unwrap();
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].relationship_type, RelationshipType::Inherits);
        
        // Get symbol relationships (both directions)
        let (outgoing, incoming) = repo.get_symbol_relationships(element2_id).unwrap();
        assert_eq!(outgoing.len(), 1); // ClassB inherits from ClassA
        assert_eq!(incoming.len(), 0);
        
        let (outgoing, incoming) = repo.get_symbol_relationships(element1_id).unwrap();
        assert_eq!(outgoing.len(), 0);
        assert_eq!(incoming.len(), 1); // ClassA is inherited by ClassB
        
        // Delete
        let relationship_id = created_relationship.id.unwrap();
        repo.delete_symbol_relationship(relationship_id).unwrap();
        
        let empty_relationships = repo.query_symbol_relationships(&query).unwrap();
        assert_eq!(empty_relationships.len(), 0);
    }

    #[test]
    fn test_mcp_session_crud() {
        let repo = create_test_repository();
        
        // Create session
        let session = McpQuerySession::new("Claude".to_string());
        let session_id = session.session_id;
        
        let created_session = repo.create_mcp_session(session).unwrap();
        assert_eq!(created_session.client_name, "Claude");
        
        // Read by ID
        let retrieved_session = repo.get_mcp_session(&session_id).unwrap().unwrap();
        assert_eq!(retrieved_session.client_name, "Claude");
        assert_eq!(retrieved_session.status, SessionStatus::Active);
        
        // Query sessions
        let query = SessionQuery::new().with_client("Claude".to_string());
        let sessions = repo.query_mcp_sessions(&query).unwrap();
        assert_eq!(sessions.len(), 1);
        
        // Update
        let mut updated_session = retrieved_session;
        updated_session.query_count = 5;
        repo.update_mcp_session(&updated_session).unwrap();
        
        let retrieved_updated = repo.get_mcp_session(&session_id).unwrap().unwrap();
        assert_eq!(retrieved_updated.query_count, 5);
        
        // Delete
        repo.delete_mcp_session(&session_id).unwrap();
        assert!(repo.get_mcp_session(&session_id).unwrap().is_none());
    }

    #[test]
    fn test_index_statistics() {
        let repo = create_test_repository();
        
        // Create index
        let index = CodeIndex::new("Test Index".to_string(), "/test/path".to_string());
        let index_id = index.id;
        repo.create_code_index(index).unwrap();
        
        // Add some data
        let metadata = FileMetadata::new(
            index_id,
            "src/test.cpp".to_string(),
            "a".repeat(64),
            Utc::now(),
            1024,
        );
        repo.create_file_metadata(metadata).unwrap();
        
        let element = CodeElement::new(
            index_id,
            "testFunction".to_string(),
            SymbolType::Function,
            "src/test.cpp".to_string(),
            10,
            5,
            "a".repeat(64),
        );
        repo.create_code_element(element).unwrap();
        
        // Get statistics
        let stats = repo.get_index_statistics().unwrap();
        assert!(stats.contains_key("Test Index"));
        
        let test_stats = &stats["Test Index"];
        assert_eq!(test_stats.actual_files, 1);
        assert_eq!(test_stats.actual_elements, 1);
        assert_eq!(test_stats.relationships, 0);
    }
}