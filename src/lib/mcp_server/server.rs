use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

// TODO: Enable when repository interface is finalized
// use crate::lib::storage::repository::Repository;
use super::tool_handlers::ToolHandlers;
use super::resource_handlers::ResourceHandlers;
use super::transport::Transport;

/// MCP Protocol Implementation
/// 
/// Implements the Model Context Protocol server specification for serving
/// C++ codebase indices. Handles initialization, capabilities negotiation,
/// tool calls, and resource requests over STDIO transport.
#[derive(Debug)]
pub struct McpServer {
    /// Server information
    info: ServerInfo,
    /// Available capabilities
    capabilities: ServerCapabilities,
    /// Tool handlers for MCP tools
    tool_handlers: ToolHandlers,
    /// Resource handlers for MCP resources
    resource_handlers: ResourceHandlers,
    /// Transport layer for message handling
    transport: Transport,
    // TODO: Add database repository when available
    // repository: Repository,
    /// Active sessions
    sessions: HashMap<String, McpSession>,
}

/// Server information sent during initialization
#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: String,
}

/// Server capabilities for MCP protocol
#[derive(Debug, Clone, Serialize)]
pub struct ServerCapabilities {
    pub tools: Vec<ToolCapability>,
    pub resources: Vec<ResourceCapability>,
    pub prompts: Vec<PromptCapability>,
}

/// Tool capability definition
#[derive(Debug, Clone, Serialize)]
pub struct ToolCapability {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Resource capability definition
#[derive(Debug, Clone, Serialize)]
pub struct ResourceCapability {
    pub uri: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub name: String,
    pub description: String,
}

/// Prompt capability definition
#[derive(Debug, Clone, Serialize)]
pub struct PromptCapability {
    pub name: String,
    pub description: String,
}

/// MCP Session state
#[derive(Debug)]
pub struct McpSession {
    pub id: String,
    pub client_info: Option<ClientInfo>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

/// Client information received during initialization
#[derive(Debug, Clone, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// MCP Protocol Message Types
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "method")]
pub enum McpRequest {
    #[serde(rename = "initialize")]
    Initialize {
        id: Value,
        params: InitializeParams,
    },
    #[serde(rename = "tools/call")]
    ToolsCall {
        id: Value,
        params: ToolCallParams,
    },
    #[serde(rename = "resources/read")]
    ResourcesRead {
        id: Value,
        params: ResourceReadParams,
    },
    #[serde(rename = "resources/list")]
    ResourcesList {
        id: Value,
        params: Option<Value>,
    },
    #[serde(rename = "tools/list")]
    ToolsList {
        id: Value,
        params: Option<Value>,
    },
    #[serde(rename = "prompts/list")]
    PromptsList {
        id: Value,
        params: Option<Value>,
    },
    #[serde(rename = "ping")]
    Ping {
        id: Value,
        params: Option<Value>,
    },
}

/// Initialize request parameters
#[derive(Debug, Clone, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
    pub capabilities: Option<Value>,
}

/// Tool call request parameters
#[derive(Debug, Clone, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    pub arguments: Value,
}

/// Resource read request parameters
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceReadParams {
    pub uri: String,
}

/// MCP Response message
#[derive(Debug, Clone, Serialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

/// MCP Error response
#[derive(Debug, Clone, Serialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl McpServer {
    /// Create new MCP server instance
    pub fn new() -> Result<Self> {
        let info = ServerInfo {
            name: "cpp-index-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "C++ codebase indexing MCP server with Tree-sitter and LibClang support".to_string(),
            author: "Generated with Claude Code".to_string(),
            homepage: "https://github.com/anthropics/claude-code".to_string(),
        };

        let capabilities = Self::build_capabilities()?;
        let tool_handlers = ToolHandlers::new()?;
        let resource_handlers = ResourceHandlers::new()?;
        let transport = Transport::new()?;

        Ok(Self {
            info,
            capabilities,
            tool_handlers,
            resource_handlers,
            transport,
            // repository,
            sessions: HashMap::new(),
        })
    }

    /// Build server capabilities from tool and resource specifications
    fn build_capabilities() -> Result<ServerCapabilities> {
        // Load tool specifications from embedded JSON
        let tools_json = include_str!("../../../specs/001-build-a-codebase/contracts/mcp-tools.json");
        let tools_spec: Value = serde_json::from_str(tools_json)?;
        
        let tools = tools_spec["tools"]
            .as_array()
            .ok_or_else(|| anyhow!("Invalid tools specification"))?
            .iter()
            .map(|tool| ToolCapability {
                name: tool["name"].as_str().unwrap().to_string(),
                description: tool["description"].as_str().unwrap().to_string(),
                input_schema: tool["inputSchema"].clone(),
            })
            .collect();

        // Define available resources
        let resources = vec![
            ResourceCapability {
                uri: "index://metadata".to_string(),
                mime_type: "application/json".to_string(),
                name: "Index Metadata".to_string(),
                description: "Metadata about available code indices".to_string(),
            },
            ResourceCapability {
                uri: "index://schema".to_string(),
                mime_type: "application/json".to_string(),
                name: "Database Schema".to_string(),
                description: "SQLite database schema information".to_string(),
            },
        ];

        let prompts = vec![];

        Ok(ServerCapabilities {
            tools,
            resources,
            prompts,
        })
    }

    /// Start the MCP server
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting MCP server: {}", self.info.name);
        
        let (tx, mut rx) = mpsc::channel::<McpRequest>(100);
        
        // Start transport layer
        self.transport.start(tx).await?;

        // Main message processing loop
        while let Some(request) = rx.recv().await {
            match self.handle_request(request).await {
                Ok(response) => {
                    if let Err(e) = self.transport.send_response(response).await {
                        error!("Failed to send response: {}", e);
                    }
                }
                Err(e) => {
                    error!("Request handling failed: {}", e);
                    // Send error response if possible
                }
            }
        }

        Ok(())
    }

    /// Handle incoming MCP requests
    #[instrument(skip(self))]
    async fn handle_request(&mut self, request: McpRequest) -> Result<McpResponse> {
        match request {
            McpRequest::Initialize { id, params } => {
                self.handle_initialize(id, params).await
            }
            McpRequest::ToolsCall { id, params } => {
                self.handle_tools_call(id, params).await
            }
            McpRequest::ResourcesRead { id, params } => {
                self.handle_resources_read(id, params).await
            }
            McpRequest::ResourcesList { id, .. } => {
                self.handle_resources_list(id).await
            }
            McpRequest::ToolsList { id, .. } => {
                self.handle_tools_list(id).await
            }
            McpRequest::PromptsList { id, .. } => {
                self.handle_prompts_list(id).await
            }
            McpRequest::Ping { id, .. } => {
                self.handle_ping(id).await
            }
        }
    }

    /// Handle initialization request
    #[instrument(skip(self))]
    async fn handle_initialize(&mut self, id: Value, params: InitializeParams) -> Result<McpResponse> {
        info!("Initializing session with client: {} v{}", 
              params.client_info.name, params.client_info.version);

        // Validate protocol version
        if params.protocol_version != "2024-11-05" {
            warn!("Unsupported protocol version: {}", params.protocol_version);
        }

        // Create new session
        let session_id = Uuid::new_v4().to_string();
        let session = McpSession {
            id: session_id.clone(),
            client_info: Some(params.client_info),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
        };
        
        self.sessions.insert(session_id, session);

        // Send initialization response
        let result = json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": self.info,
            "capabilities": self.capabilities
        });

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        })
    }

    /// Handle tool call request
    #[instrument(skip(self))]
    async fn handle_tools_call(&mut self, id: Value, params: ToolCallParams) -> Result<McpResponse> {
        info!("Handling tool call: {}", params.name);
        
        match self.tool_handlers.handle_tool_call(&params.name, params.arguments).await {
            Ok(result) => Ok(McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(result),
                error: None,
            }),
            Err(e) => {
                error!("Tool call failed: {}", e);
                Ok(McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(McpError {
                        code: -32603, // Internal error
                        message: format!("Tool execution failed: {}", e),
                        data: None,
                    }),
                })
            }
        }
    }

    /// Handle resource read request
    #[instrument(skip(self))]
    async fn handle_resources_read(&mut self, id: Value, params: ResourceReadParams) -> Result<McpResponse> {
        info!("Reading resource: {}", params.uri);
        
        match self.resource_handlers.handle_resource_read(&params.uri).await {
            Ok(result) => Ok(McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(result),
                error: None,
            }),
            Err(e) => {
                error!("Resource read failed: {}", e);
                Ok(McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(McpError {
                        code: -32603,
                        message: format!("Resource read failed: {}", e),
                        data: None,
                    }),
                })
            }
        }
    }

    /// Handle resources list request
    #[instrument(skip(self))]
    async fn handle_resources_list(&self, id: Value) -> Result<McpResponse> {
        let result = json!({
            "resources": self.capabilities.resources
        });

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        })
    }

    /// Handle tools list request
    #[instrument(skip(self))]
    async fn handle_tools_list(&self, id: Value) -> Result<McpResponse> {
        let result = json!({
            "tools": self.capabilities.tools
        });

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        })
    }

    /// Handle prompts list request
    #[instrument(skip(self))]
    async fn handle_prompts_list(&self, id: Value) -> Result<McpResponse> {
        let result = json!({
            "prompts": self.capabilities.prompts
        });

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        })
    }

    /// Handle ping request
    #[instrument(skip(self))]
    async fn handle_ping(&self, id: Value) -> Result<McpResponse> {
        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({})),
            error: None,
        })
    }

    /// Cleanup expired sessions
    pub fn cleanup_sessions(&mut self) {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
        self.sessions.retain(|_, session| session.last_activity > cutoff);
    }

    /// Get active session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use tempfile::TempDir; // TODO: Enable when needed

    #[tokio::test]
    async fn test_server_creation() {
        let server = McpServer::new().unwrap();
        
        assert_eq!(server.info.name, "cpp-index-mcp");
        assert!(!server.capabilities.tools.is_empty());
        assert_eq!(server.session_count(), 0);
    }

    #[tokio::test] 
    async fn test_capabilities_building() {
        let capabilities = McpServer::build_capabilities().unwrap();
        
        // Should have all 8 MCP tools
        assert_eq!(capabilities.tools.len(), 8);
        
        // Should have expected tool names
        let tool_names: Vec<&str> = capabilities.tools.iter()
            .map(|t| t.name.as_str())
            .collect();
        
        assert!(tool_names.contains(&"index_codebase"));
        assert!(tool_names.contains(&"search_symbols"));
        assert!(tool_names.contains(&"get_symbol_details"));
        assert!(tool_names.contains(&"find_references"));
        assert!(tool_names.contains(&"list_indices"));
        assert!(tool_names.contains(&"delete_index"));
        assert!(tool_names.contains(&"get_file_symbols"));
        assert!(tool_names.contains(&"update_file"));
    }
}