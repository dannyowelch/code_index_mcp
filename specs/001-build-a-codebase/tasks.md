# Tasks: C++ Codebase Index MCP Server

**Input**: Design documents from `/specs/001-build-a-codebase/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md

## Execution Flow (main)
```
1. Load plan.md from feature directory
   → Tech stack: Rust 1.75+, SQLite (rusqlite), Tree-sitter-cpp, LibClang, MCP SDK, Tokio
   → Libraries: cpp-indexer, mcp-server, storage, cli-interface
   → Structure: Single project with src/lib/ modules
2. Load optional design documents:
   → data-model.md: 5 entities → Code Index, Code Element, File Metadata, Symbol Relationships, MCP Query Session
   → contracts/: 3 files → mcp-tools.json (8 tools), mcp-resources.json, response-schemas.json
   → quickstart.md: 6 test scenarios → Installation, indexing, menu, MCP, incremental, performance
3. Generate tasks by category:
   → Setup: Rust project, dependencies, test data
   → Tests: contract tests for 8 MCP tools, integration tests for 6 scenarios
   → Core: 5 data models, 4 library modules, CLI commands
   → Integration: SQLite schema, MCP server, file watching
   → Polish: unit tests, performance validation, documentation
4. Apply task rules:
   → Different files = mark [P] for parallel
   → Tests before implementation (TDD strict)
   → Library modules are independent [P]
5. Number tasks sequentially (T001-T042)
6. Validate: All 8 MCP tools have tests, all 5 entities have models, TDD ordering enforced
```

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Path Conventions
Single project structure at repository root:
- **Libraries**: `src/lib/cpp_indexer/`, `src/lib/mcp_server/`, `src/lib/storage/`, `src/lib/cli_interface/`
- **Main**: `src/main.rs`, `src/config.rs`  
- **Tests**: `tests/contract/`, `tests/integration/`, `tests/unit/`
- **Data**: `test-data/sample-cpp/`

## Phase 3.1: Setup
- [x] T001 Create Rust project structure with Cargo.toml and library modules
- [x] T002 Initialize Cargo workspace with dependencies (rusqlite, tree-sitter, clang-sys, tokio, mcp-rust-sdk)
- [x] T003 [P] Configure clippy, rustfmt, and .gitignore for Rust project
- [x] T004 [P] Create test-data/sample-cpp/ directory with sample C++ files for validation

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**

### Contract Tests (MCP Tools)
- [x] T005 [P] Contract test index_codebase tool in tests/contract/test_index_codebase.rs
- [x] T006 [P] Contract test search_symbols tool in tests/contract/test_search_symbols.rs  
- [x] T007 [P] Contract test get_symbol_details tool in tests/contract/test_symbol_details.rs
- [x] T008 [P] Contract test find_references tool in tests/contract/test_find_references.rs
- [x] T009 [P] Contract test list_indices tool in tests/contract/test_list_indices.rs
- [x] T010 [P] Contract test delete_index tool in tests/contract/test_delete_index.rs
- [x] T011 [P] Contract test get_file_symbols tool in tests/contract/test_file_symbols.rs
- [x] T012 [P] Contract test update_file tool in tests/contract/test_update_file.rs

### Integration Tests (Quickstart Scenarios)
- [x] T013 [P] Integration test system installation & setup in tests/integration/test_installation.rs
- [x] T014 [P] Integration test sample index creation in tests/integration/test_sample_indexing.rs
- [x] T015 [P] Integration test menu interface validation in tests/integration/test_menu_interface.rs
- [x] T016 [P] Integration test MCP protocol integration in tests/integration/test_mcp_protocol.rs
- [x] T017 [P] Integration test incremental update functionality in tests/integration/test_incremental_update.rs
- [x] T018 [P] Integration test performance validation in tests/integration/test_performance.rs

## Phase 3.3: Data Models (ONLY after tests are failing)
- [ ] T019 [P] Code Index model in src/lib/storage/models/code_index.rs
- [ ] T020 [P] Code Element model in src/lib/storage/models/code_element.rs
- [ ] T021 [P] File Metadata model in src/lib/storage/models/file_metadata.rs
- [ ] T022 [P] Symbol Relationships model in src/lib/storage/models/symbol_relationships.rs
- [ ] T023 [P] MCP Query Session model in src/lib/storage/models/mcp_query_session.rs

## Phase 3.4: Core Library Implementation
### Storage Library
- [ ] T024 [P] SQLite schema migrations in src/lib/storage/schema.rs
- [ ] T025 [P] Database connection and configuration in src/lib/storage/connection.rs
- [ ] T026 [P] CRUD operations for all models in src/lib/storage/repository.rs

### C++ Indexer Library  
- [ ] T027 [P] Tree-sitter C++ parser integration in src/lib/cpp_indexer/tree_sitter_parser.rs
- [ ] T028 [P] LibClang semantic analysis integration in src/lib/cpp_indexer/clang_parser.rs
- [ ] T029 [P] Symbol extraction and processing in src/lib/cpp_indexer/symbol_extractor.rs
- [ ] T030 [P] Incremental indexing with Merkle tree in src/lib/cpp_indexer/incremental.rs

### MCP Server Library
- [ ] T031 [P] MCP protocol server implementation in src/lib/mcp_server/server.rs
- [ ] T032 [P] Tool handlers for all 8 MCP tools in src/lib/mcp_server/tool_handlers.rs
- [ ] T033 [P] Resource handlers for MCP resources in src/lib/mcp_server/resource_handlers.rs
- [ ] T034 [P] STDIO transport and message routing in src/lib/mcp_server/transport.rs

### CLI Interface Library
- [ ] T035 [P] Interactive menu system in src/lib/cli_interface/menu.rs
- [ ] T036 [P] Command-line argument parsing in src/lib/cli_interface/cli_args.rs
- [ ] T037 [P] User input validation and prompts in src/lib/cli_interface/user_input.rs

## Phase 3.5: Application Integration
- [ ] T038 Main application entry point and command routing in src/main.rs
- [ ] T039 Configuration management and settings in src/config.rs
- [ ] T040 Structured logging with file paths and error context
- [ ] T041 Cross-platform file handling and path normalization (Windows focus)

## Phase 3.6: Polish & Validation
- [ ] T042 [P] Unit tests for symbol extraction logic in tests/unit/test_symbol_extraction.rs
- [ ] T043 [P] Unit tests for incremental indexing in tests/unit/test_incremental_logic.rs
- [ ] T044 [P] Unit tests for MCP message handling in tests/unit/test_mcp_messages.rs
- [ ] T045 Performance benchmarks with large codebase (>10k files) in tests/performance/
- [ ] T046 [P] Memory usage profiling during indexing in tests/performance/memory_profile.rs
- [ ] T047 [P] Query response time validation (<100ms) in tests/performance/query_benchmarks.rs
- [ ] T048 [P] Update CLAUDE.md with final implementation details
- [ ] T049 Execute complete quickstart.md validation scenarios
- [ ] T050 [P] Create llms.txt documentation for each library module

## Dependencies
**Phase Dependencies:**
- Setup (T001-T004) before all other phases
- Tests (T005-T018) before implementation (T019-T041) - **TDD NON-NEGOTIABLE**
- Data models (T019-T023) before repositories (T026) and tool handlers (T032)
- Core libraries (T024-T037) before application integration (T038-T041)
- Implementation (T019-T041) before polish (T042-T050)

**Specific Blocking Dependencies:**
- T024 (schema) blocks T025 (connection) and T026 (repository)
- T019-T023 (models) block T026 (repository) and T032 (tool handlers)
- T027-T030 (indexer) block T032 (tool handlers)
- T031-T034 (MCP server) block T038 (main)
- T035-T037 (CLI) block T038 (main)

## Parallel Execution Examples

### Phase 3.2: Launch all contract tests together
```bash
# All contract tests can run in parallel (different files)
Task: "Contract test index_codebase tool in tests/contract/test_index_codebase.rs"
Task: "Contract test search_symbols tool in tests/contract/test_search_symbols.rs"  
Task: "Contract test get_symbol_details tool in tests/contract/test_symbol_details.rs"
Task: "Contract test find_references tool in tests/contract/test_find_references.rs"
Task: "Contract test list_indices tool in tests/contract/test_list_indices.rs"
Task: "Contract test delete_index tool in tests/contract/test_delete_index.rs"
Task: "Contract test get_file_symbols tool in tests/contract/test_file_symbols.rs"
Task: "Contract test update_file tool in tests/contract/test_update_file.rs"
```

### Phase 3.3: Launch all data models together
```bash
# All model files are independent
Task: "Code Index model in src/lib/storage/models/code_index.rs"
Task: "Code Element model in src/lib/storage/models/code_element.rs"
Task: "File Metadata model in src/lib/storage/models/file_metadata.rs"
Task: "Symbol Relationships model in src/lib/storage/models/symbol_relationships.rs"
Task: "MCP Query Session model in src/lib/storage/models/mcp_query_session.rs"
```

### Phase 3.4: Launch library modules by category
```bash
# Storage library tasks (after models complete)
Task: "SQLite schema migrations in src/lib/storage/schema.rs"
Task: "Database connection and configuration in src/lib/storage/connection.rs"
Task: "CRUD operations for all models in src/lib/storage/repository.rs"

# C++ Indexer library tasks (all parallel)
Task: "Tree-sitter C++ parser integration in src/lib/cpp_indexer/tree_sitter_parser.rs"
Task: "LibClang semantic analysis integration in src/lib/cpp_indexer/clang_parser.rs"  
Task: "Symbol extraction and processing in src/lib/cpp_indexer/symbol_extractor.rs"
Task: "Incremental indexing with Merkle tree in src/lib/cpp_indexer/incremental.rs"
```

## Task Generation Rules Applied
1. **From Contracts**: 8 MCP tools → 8 contract tests (T005-T012) [P]
2. **From Data Model**: 5 entities → 5 model tasks (T019-T023) [P]
3. **From Quickstart**: 6 scenarios → 6 integration tests (T013-T018) [P] 
4. **From Architecture**: 4 libraries → module implementation tasks grouped by library

## Validation Checklist ✅
- [x] All 8 MCP tool contracts have corresponding tests (T005-T012)
- [x] All 5 entities have model creation tasks (T019-T023)
- [x] All tests come before implementation (Phase 3.2 before 3.3+)
- [x] Parallel tasks truly independent (different files, no shared state)
- [x] Each task specifies exact file path for clarity
- [x] No task modifies same file as another [P] task
- [x] TDD ordering strictly enforced with phase gates
- [x] Performance requirements covered in validation tasks
- [x] Cross-platform Windows support addressed

## Notes
- **[P] tasks** = different files, no dependencies, can run concurrently
- **Verify tests fail** before implementing - RED phase is mandatory
- **Commit after each task** to maintain TDD git history
- **Performance targets**: 100k+ files, <30s incremental updates, <100ms queries
- **Windows focus**: File path handling, executable deployment
- **Memory efficiency**: Streaming parsers, bounded queues, memory mapping