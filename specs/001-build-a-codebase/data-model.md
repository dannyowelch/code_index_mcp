# Data Model: C++ Codebase Index MCP Server

**Phase**: 1 - Design & Contracts  
**Date**: 2025-09-07  
**Based on**: Feature specification and research findings

## Core Entities

### 1. Code Index
**Purpose**: Represents a complete searchable index for a C++ codebase
**Fields**:
- `id`: Unique identifier (UUID)
- `name`: Human-readable name (e.g., "Unreal Engine 5.3")
- `base_path`: Root directory path of the indexed codebase
- `created_at`: Timestamp of index creation
- `updated_at`: Timestamp of last index update
- `total_files`: Count of indexed files
- `total_symbols`: Count of indexed symbols
- `index_version`: Schema version for migration support

**Relationships**: One-to-many with Index Entries

**Validation Rules**:
- `base_path` must be valid, accessible directory
- `name` must be unique within the system
- `total_files` and `total_symbols` must be non-negative

### 2. Code Element
**Purpose**: Individual C++ code symbols and constructs
**Fields**:
- `id`: Unique identifier (auto-increment)
- `index_id`: Foreign key to Code Index
- `symbol_name`: Name of the symbol (function, class, variable)
- `symbol_type`: Type of symbol (function, class, variable, macro, namespace, enum, typedef)
- `file_path`: Relative path from codebase root
- `line_number`: Line number in file (1-based)
- `column_number`: Column number in file (1-based)
- `definition_hash`: Blake3 hash of definition for change detection
- `scope`: Fully qualified scope (e.g., "MyNamespace::MyClass")
- `access_modifier`: public, private, protected, or null
- `is_declaration`: Boolean - true if declaration, false if definition
- `signature`: Function signature or variable type (optional)

**Relationships**: 
- Many-to-one with Code Index
- Self-referencing for inheritance/usage relationships

**Validation Rules**:
- `file_path` must be relative path with forward slashes
- `line_number` and `column_number` must be positive
- `symbol_type` must be from predefined enum
- `definition_hash` must be valid Blake3 hash (64 chars hex)

### 3. File Metadata
**Purpose**: Tracks file-level information for incremental updates
**Fields**:
- `id`: Unique identifier (auto-increment)
- `index_id`: Foreign key to Code Index
- `file_path`: Relative path from codebase root
- `file_hash`: Blake3 hash of entire file content
- `last_modified`: File system modification time
- `size_bytes`: File size in bytes
- `symbol_count`: Number of symbols in this file
- `indexed_at`: Timestamp when file was last indexed

**Relationships**: Many-to-one with Code Index

**Validation Rules**:
- `file_path` must be unique within an index
- `size_bytes` and `symbol_count` must be non-negative
- `file_hash` must be valid Blake3 hash

### 4. Symbol Relationships
**Purpose**: Tracks relationships between code elements (inheritance, usage, includes)
**Fields**:
- `id`: Unique identifier (auto-increment)
- `from_symbol_id`: Foreign key to Code Element (source)
- `to_symbol_id`: Foreign key to Code Element (target)
- `relationship_type`: Type of relationship (inherits, uses, includes, calls, defines)
- `file_path`: File where relationship occurs
- `line_number`: Line number where relationship is declared

**Relationships**: Many-to-many linking Code Elements

**Validation Rules**:
- Both symbol IDs must exist and belong to same index
- `relationship_type` must be from predefined enum
- `from_symbol_id` and `to_symbol_id` must be different

### 5. MCP Query Session
**Purpose**: Tracks MCP client sessions and query history
**Fields**:
- `session_id`: Unique session identifier (UUID)
- `client_name`: Name of AI assistant client
- `active_index_id`: Currently active Code Index
- `created_at`: Session start time
- `last_activity`: Last query timestamp
- `query_count`: Number of queries in session

**Relationships**: Many-to-one with Code Index (via active_index_id)

**Validation Rules**:
- `session_id` must be unique
- `query_count` must be non-negative
- `active_index_id` must reference valid Code Index

## State Transitions

### Code Index Lifecycle
```
[Creating] → [Active] → [Updating] → [Active]
          ↓          ↓           ↓
        [Failed]   [Archived] [Failed]
```

**States**:
- **Creating**: Index is being built for the first time
- **Active**: Index is complete and queryable
- **Updating**: Incremental update in progress
- **Archived**: Index preserved but not actively maintained
- **Failed**: Index creation or update failed

### File Processing States
```
[Pending] → [Processing] → [Indexed]
         ↓              ↓
       [Error]      [Error]
```

## Database Schema Optimizations

### Indexing Strategy
```sql
-- Primary indexes for fast lookups
CREATE INDEX idx_symbols_name ON code_elements(symbol_name);
CREATE INDEX idx_symbols_file_path ON code_elements(file_path);
CREATE INDEX idx_symbols_type ON code_elements(symbol_type);
CREATE INDEX idx_symbols_scope ON code_elements(scope);

-- Composite indexes for common query patterns
CREATE INDEX idx_symbols_file_line ON code_elements(file_path, line_number);
CREATE INDEX idx_symbols_name_type ON code_elements(symbol_name, symbol_type);

-- Hash indexes for change detection
CREATE INDEX idx_file_hash ON file_metadata(file_hash);
CREATE INDEX idx_definition_hash ON code_elements(definition_hash);
```

### Partitioning Strategy
For large codebases (millions of symbols), partition tables by:
- **Index ID**: Separate table per codebase
- **File path prefix**: Separate modules/directories
- **Symbol type**: Functions, classes, variables in separate tables

## Storage Efficiency

### Hash-based Deduplication
- Use Blake3 hashes to detect unchanged files and symbols
- Store only hash references for duplicate signatures
- Compress common strings (file paths, namespaces) using dictionary encoding

### Memory Layout
- Store frequently accessed fields (name, file, line) together
- Separate blob storage for large signatures and documentation
- Use integer IDs for foreign key relationships

## Migration Strategy

### Version Evolution
- Schema version tracked in `code_index.index_version`
- Backward compatibility maintained for 2 major versions
- Migration scripts for schema upgrades

### Data Integrity
- Foreign key constraints enforced
- Check constraints for enum values
- Cascade deletes for index removal