//! C++ Codebase Index MCP Server Library
//! 
//! This library provides fast, efficient indexing of large C++ codebases
//! with MCP (Model Context Protocol) support for AI coding assistants.

pub mod config;

// Re-export main modules for easy access
pub use config::Config;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");