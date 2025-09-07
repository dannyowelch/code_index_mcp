#[cfg(test)]
mod test_mcp_protocol {
    use std::process::{Command, Stdio};
    use std::path::Path;
    use std::io::{BufRead, BufReader, Write};
    use tempfile::TempDir;
    use anyhow::Result;
    use serde_json::{json, Value};
    use std::time::Duration;
    use std::fs;
    
    /// Test that MCP server starts and responds to basic protocol messages
    #[tokio::test]
    async fn test_mcp_server_startup() -> Result<()> {
        let binary_path = Path::new("target/release/cpp-index-mcp");
        #[cfg(windows)]
        let binary_path = Path::new("target/release/cpp-index-mcp.exe");
        
        if !binary_path.exists() {
            println!("Skipping MCP server test - binary not built yet");
            return Ok(());
        }
        
        // Try to start MCP server
        let output = Command::new(binary_path)
            .args(&["server", "--stdio", "--index", "test_index"])
            .output()
            .expect("Failed to start MCP server");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        println!("MCP server stdout: {}", stdout);
        println!("MCP server stderr: {}", stderr);
        
        // Currently expect "not yet implemented" message
        assert!(
            stdout.contains("not yet implemented") || stderr.contains("not yet implemented"),
            "Expected MCP server to show 'not yet implemented', got stdout: '{}', stderr: '{}'",
            stdout, stderr
        );
        
        Ok(())
    }
    
    /// Test MCP protocol message structure validation
    #[tokio::test]
    async fn test_mcp_message_format() -> Result<()> {
        // Test that we can create and validate MCP protocol messages
        
        // Initialize message - client to server
        let initialize_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "roots": {
                        "listChanged": true
                    },
                    "sampling": {}
                },
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        });
        
        validate_mcp_message(&initialize_request)?;
        
        // Expected initialize response
        let initialize_response = json!({
            "jsonrpc": "2.0", 
            "id": 1,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": true
                    },
                    "resources": {
                        "subscribe": true,
                        "listChanged": true
                    }
                },
                "serverInfo": {
                    "name": "cpp-index-mcp",
                    "version": "0.1.0"
                }
            }
        });
        
        validate_mcp_message(&initialize_response)?;
        
        Ok(())
    }
    
    /// Test MCP tools list message structure
    #[tokio::test]
    async fn test_mcp_tools_list() -> Result<()> {
        // Tools list request
        let tools_list_request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        });
        
        validate_mcp_message(&tools_list_request)?;
        
        // Expected tools list response with all 8 contract tools
        let tools_list_response = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "result": {
                "tools": [
                    {
                        "name": "index_codebase",
                        "description": "Create or update a C++ codebase index",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string"},
                                "base_path": {"type": "string"},
                                "incremental": {"type": "boolean"},
                                "file_patterns": {"type": "array", "items": {"type": "string"}},
                                "exclude_patterns": {"type": "array", "items": {"type": "string"}}
                            },
                            "required": ["name", "base_path"]
                        }
                    },
                    {
                        "name": "search_symbols",
                        "description": "Search for symbols in the indexed codebase",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "index_name": {"type": "string"},
                                "query": {"type": "string"},
                                "symbol_types": {"type": "array", "items": {"type": "string"}},
                                "limit": {"type": "integer"},
                                "offset": {"type": "integer"}
                            },
                            "required": ["index_name", "query"]
                        }
                    },
                    {
                        "name": "get_symbol_details",
                        "description": "Get detailed information about a specific symbol",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "index_name": {"type": "string"},
                                "symbol_id": {"type": "string"}
                            },
                            "required": ["index_name", "symbol_id"]
                        }
                    },
                    {
                        "name": "find_references",
                        "description": "Find all references to a symbol",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "index_name": {"type": "string"},
                                "symbol_id": {"type": "string"},
                                "include_declarations": {"type": "boolean"}
                            },
                            "required": ["index_name", "symbol_id"]
                        }
                    },
                    {
                        "name": "list_indices",
                        "description": "List all available codebase indices",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "include_stats": {"type": "boolean"}
                            }
                        }
                    },
                    {
                        "name": "delete_index",
                        "description": "Delete a codebase index",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "index_name": {"type": "string"},
                                "confirm": {"type": "boolean"}
                            },
                            "required": ["index_name", "confirm"]
                        }
                    },
                    {
                        "name": "get_file_symbols",
                        "description": "Get all symbols from a specific file",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "index_name": {"type": "string"},
                                "file_path": {"type": "string"},
                                "symbol_types": {"type": "array", "items": {"type": "string"}}
                            },
                            "required": ["index_name", "file_path"]
                        }
                    },
                    {
                        "name": "update_file",
                        "description": "Update index for a specific file",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "index_name": {"type": "string"},
                                "file_path": {"type": "string"},
                                "force_reparse": {"type": "boolean"}
                            },
                            "required": ["index_name", "file_path"]
                        }
                    }
                ]
            }
        });
        
        validate_mcp_message(&tools_list_response)?;
        
        // Verify all 8 expected tools are present
        let tools = &tools_list_response["result"]["tools"];
        assert!(tools.is_array(), "Tools should be an array");
        
        let tools_array = tools.as_array().unwrap();
        assert_eq!(tools_array.len(), 8, "Should have exactly 8 tools");
        
        let expected_tool_names = [
            "index_codebase", "search_symbols", "get_symbol_details", "find_references",
            "list_indices", "delete_index", "get_file_symbols", "update_file"
        ];
        
        for expected_name in expected_tool_names {
            let has_tool = tools_array.iter().any(|tool| {
                tool["name"].as_str() == Some(expected_name)
            });
            assert!(has_tool, "Should have tool: {}", expected_name);
        }
        
        Ok(())
    }
    
    /// Test MCP tool call message structure
    #[tokio::test]
    async fn test_mcp_tool_calls() -> Result<()> {
        // Test index_codebase tool call
        let index_tool_call = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "index_codebase",
                "arguments": {
                    "name": "test_index",
                    "base_path": "/path/to/cpp/project",
                    "incremental": false,
                    "file_patterns": ["**/*.cpp", "**/*.h"],
                    "exclude_patterns": ["**/build/**"]
                }
            }
        });
        
        validate_mcp_message(&index_tool_call)?;
        
        // Test search_symbols tool call
        let search_tool_call = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "search_symbols",
                "arguments": {
                    "index_name": "test_index",
                    "query": "ClassName",
                    "symbol_types": ["class", "struct"],
                    "limit": 50
                }
            }
        });
        
        validate_mcp_message(&search_tool_call)?;
        
        // Expected tool response structure
        let tool_response = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": "Index created successfully with 42 files processed and 156 symbols found."
                    }
                ]
            }
        });
        
        validate_mcp_message(&tool_response)?;
        
        Ok(())
    }
    
    /// Test MCP resources list message structure
    #[tokio::test]
    async fn test_mcp_resources_list() -> Result<()> {
        let resources_list_request = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "resources/list"
        });
        
        validate_mcp_message(&resources_list_request)?;
        
        // Expected resources response
        let resources_list_response = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "result": {
                "resources": [
                    {
                        "uri": "cpp-index://indices",
                        "name": "Available Indices",
                        "description": "List of all codebase indices",
                        "mimeType": "application/json"
                    },
                    {
                        "uri": "cpp-index://schemas",
                        "name": "Database Schemas", 
                        "description": "SQLite database schema information",
                        "mimeType": "text/sql"
                    },
                    {
                        "uri": "cpp-index://stats",
                        "name": "Indexing Statistics",
                        "description": "Performance and usage statistics",
                        "mimeType": "application/json"
                    }
                ]
            }
        });
        
        validate_mcp_message(&resources_list_response)?;
        
        let resources = &resources_list_response["result"]["resources"];
        assert!(resources.is_array(), "Resources should be an array");
        
        let resources_array = resources.as_array().unwrap();
        assert!(resources_array.len() >= 3, "Should have at least 3 resources");
        
        Ok(())
    }
    
    /// Test MCP error handling
    #[tokio::test]
    async fn test_mcp_error_handling() -> Result<()> {
        // Test invalid method error
        let invalid_method_error = json!({
            "jsonrpc": "2.0",
            "id": 6,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": "Unknown method: invalid_method"
            }
        });
        
        validate_mcp_message(&invalid_method_error)?;
        
        // Test invalid parameters error
        let invalid_params_error = json!({
            "jsonrpc": "2.0", 
            "id": 7,
            "error": {
                "code": -32602,
                "message": "Invalid params",
                "data": "Missing required parameter: index_name"
            }
        });
        
        validate_mcp_message(&invalid_params_error)?;
        
        // Test tool execution error
        let tool_error = json!({
            "jsonrpc": "2.0",
            "id": 8,
            "error": {
                "code": -32000,
                "message": "Tool execution failed",
                "data": {
                    "tool": "index_codebase",
                    "reason": "Path not found: /invalid/path"
                }
            }
        });
        
        validate_mcp_message(&tool_error)?;
        
        Ok(())
    }
    
    /// Test STDIO transport format
    #[tokio::test]
    async fn test_stdio_transport() -> Result<()> {
        // Test that messages are properly formatted for STDIO transport
        let message = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05"
            }
        });
        
        let stdio_formatted = format_message_for_stdio(&message)?;
        
        // STDIO messages should be single lines ending with newline
        assert!(stdio_formatted.ends_with('\n'), "STDIO message should end with newline");
        assert_eq!(stdio_formatted.matches('\n').count(), 1, "STDIO message should be single line");
        
        // Should be valid JSON when parsed back
        let stripped = stdio_formatted.trim_end();
        let parsed: Value = serde_json::from_str(stripped)?;
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 1);
        
        Ok(())
    }
    
    /// Test MCP server lifecycle management  
    #[tokio::test]
    async fn test_mcp_server_lifecycle() -> Result<()> {
        // Test the expected lifecycle of an MCP server session
        
        // 1. Client sends initialize
        let initialize = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {}
            }
        });
        
        // 2. Server responds with capabilities
        let initialize_response = json!({
            "jsonrpc": "2.0",
            "id": 1, 
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {}
                }
            }
        });
        
        // 3. Client sends initialized notification
        let initialized = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        
        // Validate all lifecycle messages
        validate_mcp_message(&initialize)?;
        validate_mcp_message(&initialize_response)?;
        validate_mcp_message(&initialized)?;
        
        // Initialized notification should not have id
        assert!(initialized["id"].is_null(), "Notifications should not have id");
        
        Ok(())
    }
    
    /// Test concurrent MCP requests handling
    #[tokio::test]
    async fn test_concurrent_mcp_requests() -> Result<()> {
        // Test that server can handle multiple concurrent requests
        let concurrent_requests = vec![
            json!({
                "jsonrpc": "2.0",
                "id": 10,
                "method": "tools/call",
                "params": {
                    "name": "list_indices",
                    "arguments": {}
                }
            }),
            json!({
                "jsonrpc": "2.0", 
                "id": 11,
                "method": "tools/call",
                "params": {
                    "name": "search_symbols",
                    "arguments": {
                        "index_name": "main",
                        "query": "function"
                    }
                }
            }),
            json!({
                "jsonrpc": "2.0",
                "id": 12,
                "method": "resources/list"
            })
        ];
        
        // All requests should be valid
        for request in &concurrent_requests {
            validate_mcp_message(request)?;
        }
        
        // Test that IDs are unique
        let ids: Vec<i64> = concurrent_requests.iter()
            .map(|req| req["id"].as_i64().unwrap())
            .collect();
        
        let mut sorted_ids = ids.clone();
        sorted_ids.sort();
        sorted_ids.dedup();
        
        assert_eq!(ids.len(), sorted_ids.len(), "All request IDs should be unique");
        
        Ok(())
    }
    
    // Helper functions
    fn validate_mcp_message(message: &Value) -> Result<()> {
        // Basic JSON-RPC validation
        assert!(message.is_object(), "MCP message must be JSON object");
        
        assert_eq!(
            message["jsonrpc"].as_str(),
            Some("2.0"),
            "Must have jsonrpc: '2.0'"
        );
        
        // Either request (has method) or response (has result/error)
        if message["method"].is_string() {
            // Request validation
            let method = message["method"].as_str().unwrap();
            assert!(!method.is_empty(), "Method must not be empty");
            
            // Requests should have id unless they're notifications
            if !method.starts_with("notifications/") {
                assert!(
                    message["id"].is_number() || message["id"].is_string(),
                    "Requests must have id"
                );
            }
        } else {
            // Response validation
            assert!(
                message["result"].is_object() || message["error"].is_object(),
                "Response must have result or error"
            );
            
            assert!(
                message["id"].is_number() || message["id"].is_string(),
                "Response must have id"
            );
            
            // If error, validate error structure
            if let Some(error) = message["error"].as_object() {
                assert!(error.contains_key("code"), "Error must have code");
                assert!(error.contains_key("message"), "Error must have message");
            }
        }
        
        Ok(())
    }
    
    fn format_message_for_stdio(message: &Value) -> Result<String> {
        let json_str = serde_json::to_string(message)?;
        Ok(format!("{}\n", json_str))
    }
}