// C++ Indexer Library
// 
// This module provides C++ code parsing and symbol extraction capabilities
// using Tree-sitter for syntax parsing and LibClang for semantic analysis.

pub mod tree_sitter_parser;
pub mod clang_parser;
pub mod symbol_extractor;
pub mod incremental;

pub use tree_sitter_parser::{TreeSitterParser, ParseResult, ParsedNode};
pub use clang_parser::{ClangParser, SemanticParseResult, SemanticInfo, SourceLocation};
pub use symbol_extractor::{SymbolExtractor, ExtractionResult, ExtractedSymbol};
pub use incremental::{IncrementalIndexer, IncrementalResult, IndexStatus, IndexAction};