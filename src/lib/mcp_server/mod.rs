// MCP Server Library
//
// This module implements the Model Context Protocol server functionality
// for serving C++ codebase indices over STDIO transport.

pub mod server;
pub mod tool_handlers;
pub mod resource_handlers;
pub mod transport;

pub use server::{McpServer, ServerInfo, ServerCapabilities};
pub use tool_handlers::ToolHandlers;
pub use resource_handlers::ResourceHandlers;
pub use transport::Transport;