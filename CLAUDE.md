# C++ Codebase Index MCP Server Development Guidelines

Auto-generated from feature plans. Last updated: 2025-09-07

## Active Technologies
- **Rust 1.75+**: Primary language for performance-critical indexing
- **SQLite (rusqlite)**: Database for efficient metadata storage
- **Tree-sitter-cpp**: Fast C++ parsing for structural analysis
- **LibClang**: Semantic analysis for complex C++ features
- **MCP Rust SDK**: Official MCP protocol implementation
- **Tokio**: Async runtime for concurrent file processing

## Project Structure
```
src/
├── lib/
│   ├── cpp-indexer/       # C++ parsing and symbol extraction
│   ├── mcp-server/        # MCP protocol implementation
│   ├── storage/           # SQLite operations and schema
│   └── cli-interface/     # Menu system and user interaction
├── main.rs                # Entry point and CLI routing
└── config.rs              # Configuration management

tests/
├── contract/              # MCP protocol contract tests
├── integration/           # End-to-end indexing tests
└── unit/                  # Library unit tests

test-data/
└── sample-cpp/            # Test C++ codebase for validation
```

## Commands
```bash
# Build and test
cargo build --release
cargo test

# Create index
./target/release/cpp-index-mcp index create --name "project" --path "/path/to/cpp"

# Launch interactive menu
./target/release/cpp-index-mcp menu

# Start MCP server
./target/release/cpp-index-mcp server --stdio --index "project"

# Query symbols
./target/release/cpp-index-mcp query --index "project" --symbol "ClassName"
```

## Code Style
- **Library-first architecture**: Every feature implemented as reusable library
- **Direct framework usage**: No wrapper abstractions over rusqlite, tree-sitter, etc.
- **Structured logging**: JSON logs with file paths, line numbers, error context
- **Error handling**: Comprehensive Result<> usage with error chain context
- **Performance focus**: Memory-mapped files, streaming parsers, bounded queues
- **Cross-platform**: Conditional compilation for Windows/Linux/macOS optimizations

## Testing Requirements (NON-NEGOTIABLE)
- **TDD mandatory**: Tests written first, must fail, then implement
- **Real dependencies**: Actual SQLite databases and C++ files, no mocks
- **Test order**: Contract → Integration → E2E → Unit
- **Git commits**: Tests committed before implementation code

## Recent Changes
- **001-build-a-codebase**: Added C++ indexing MCP server with SQLite storage, hybrid parsing (Tree-sitter + LibClang), STDIO MCP protocol, Windows support

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->