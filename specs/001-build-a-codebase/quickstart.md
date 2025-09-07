# Quickstart: C++ Codebase Index MCP Server

**Purpose**: Validate core functionality of the C++ indexing system  
**Target Audience**: Developers and AI assistants  
**Estimated Time**: 10 minutes

## Prerequisites

- Rust 1.75+ installed
- Sample C++ codebase (or use provided test data)
- AI assistant with MCP support (Claude Code, etc.)

## Quick Test Scenarios

### 1. System Installation & Setup

```bash
# Build the indexer
cargo build --release

# Verify installation
./target/release/cpp-index-mcp --version
# Expected: cpp-index-mcp 0.1.0

# Check help
./target/release/cpp-index-mcp --help
# Expected: Usage menu with available commands
```

### 2. Create Sample Index

```bash
# Create a test index with sample C++ code
./target/release/cpp-index-mcp index create \
  --name "sample-project" \
  --path "./test-data/sample-cpp" \
  --format json

# Expected output:
# {
#   "success": true,
#   "index_id": "uuid-here",
#   "files_processed": 15,
#   "symbols_found": 127,
#   "duration_ms": 1250
# }
```

**Test Data Structure:**
```
test-data/sample-cpp/
├── src/
│   ├── main.cpp          # Contains main() function
│   ├── utils.cpp         # Utility functions
│   └── math/
│       ├── calculator.h  # Calculator class declaration
│       └── calculator.cpp # Calculator implementation
├── include/
│   └── common.h          # Common defines and typedefs
└── tests/
    └── test_calculator.cpp # Unit tests
```

### 3. Menu Interface Validation

```bash
# Launch interactive menu
./target/release/cpp-index-mcp menu

# Expected menu:
# C++ Codebase Index Manager
# 1. Create new index
# 2. Update existing index  
# 3. List indices
# 4. Delete index
# 5. Query symbols
# 6. Start MCP server
# 0. Exit
# Choose option:
```

Test sequence:
1. Choose option 3 (List indices) → Should show "sample-project"
2. Choose option 5 (Query symbols) → Search for "main" → Should find main function
3. Choose option 6 (Start MCP server) → Should launch STDIO server mode

### 4. MCP Protocol Integration

**Start MCP Server:**
```bash
./target/release/cpp-index-mcp server --stdio --index "sample-project"
```

**Test MCP Tools (via AI assistant):**

```json
// Tool: search_symbols
{
  "name": "search_symbols",
  "arguments": {
    "index_name": "sample-project",
    "query": "Calculator",
    "symbol_type": "class"
  }
}

// Expected response:
{
  "symbols": [
    {
      "id": 42,
      "name": "Calculator",
      "type": "class",
      "file_path": "src/math/calculator.h",
      "line_number": 5,
      "column_number": 7,
      "scope": "",
      "signature": "class Calculator"
    }
  ],
  "total_count": 1,
  "query_time_ms": 15
}
```

```json
// Tool: get_symbol_details
{
  "name": "get_symbol_details",
  "arguments": {
    "index_name": "sample-project",
    "symbol_id": 42
  }
}

// Expected response includes relationships and full details
```

### 5. Incremental Update Test

**Modify a file:**
```bash
# Add a new function to utils.cpp
echo "void newFunction() { }" >> test-data/sample-cpp/src/utils.cpp

# Update the index
./target/release/cpp-index-mcp index update \
  --name "sample-project" \
  --file "src/utils.cpp"

# Verify the new symbol was indexed
./target/release/cpp-index-mcp query \
  --index "sample-project" \
  --symbol "newFunction"
```

### 6. Performance Validation

**Large Codebase Test (if available):**
```bash
# Test with a substantial C++ project
./target/release/cpp-index-mcp index create \
  --name "large-project" \
  --path "/path/to/large/cpp/project" \
  --stats

# Verify performance targets:
# - Files processed: >10,000
# - Indexing time: <30 seconds for incremental updates
# - Memory usage: <500MB during indexing
# - Query response: <100ms
```

## Success Criteria

### ✅ Core Functionality
- [x] Single executable builds successfully
- [x] Menu interface displays and accepts input
- [x] Can create index for C++ codebase
- [x] Finds and indexes C++ symbols (functions, classes, variables)
- [x] Stores data in SQLite database
- [x] MCP server starts and responds to STDIO

### ✅ Indexing Accuracy
- [x] Correctly identifies function definitions and declarations
- [x] Extracts class names and member functions
- [x] Handles C++ namespaces properly
- [x] Tracks file paths and line numbers accurately
- [x] Supports common C++ file extensions (.cpp, .h, .hpp)

### ✅ MCP Integration
- [x] Responds to search_symbols tool calls
- [x] Returns properly formatted JSON responses
- [x] Handles query parameters correctly
- [x] Provides symbol details with location information
- [x] Supports filtering by symbol type and file path

### ✅ Performance Targets
- [x] Handles test project (50+ files) in <5 seconds
- [x] Query responses return in <100ms
- [x] Memory usage stays under 200MB for test project
- [x] Incremental updates process only changed files

### ✅ Windows Compatibility
- [x] Handles Windows file paths correctly (backslashes)
- [x] Executable runs on Windows 10/11
- [x] File watching works for incremental updates
- [x] Database persists correctly on Windows filesystem

## Common Issues & Troubleshooting

### Issue: "Failed to parse C++ file"
**Cause**: Missing system headers or unsupported C++ features
**Solution**: Check clang installation and add include directories

### Issue: "Database locked" error
**Cause**: Multiple instances accessing same index
**Solution**: Ensure only one instance runs per index

### Issue: "Symbol not found" in search
**Cause**: Symbol may be in unindexed file or parsing failed
**Solution**: Check file inclusion patterns and parsing errors

### Issue: Slow performance on large codebases
**Cause**: Insufficient memory or disk I/O bottleneck
**Solution**: Increase available memory, use SSD storage

## Next Steps

After successful quickstart validation:
1. **Integration Testing**: Connect with your AI assistant client
2. **Performance Tuning**: Test with your actual large codebase
3. **Custom Configuration**: Adjust file patterns and parsing options
4. **Production Setup**: Configure logging and monitoring

## Validation Checklist

- [ ] Single executable builds and runs
- [ ] Menu interface works correctly  
- [ ] Index creation succeeds with test data
- [ ] MCP tools return expected responses
- [ ] Incremental updates work correctly
- [ ] Performance meets target specifications
- [ ] Windows file paths handled properly
- [ ] Query accuracy verified with known symbols

**If all items checked**: ✅ System is ready for production use with your codebase!