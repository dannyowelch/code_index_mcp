use anyhow::{anyhow, Result};
use serde_json::{json, Value};
// use std::io; // TODO: Enable when needed
use tokio::sync::mpsc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info, instrument, warn};

use super::server::{McpRequest, McpResponse};

/// STDIO Transport for MCP Protocol
/// 
/// Implements JSON-RPC 2.0 message transport over STDIO as specified by the
/// Model Context Protocol. Handles message framing, parsing, and routing
/// between the MCP client and server handlers.
#[derive(Debug)]
pub struct Transport {
    /// Channel for sending requests to server
    request_sender: Option<mpsc::Sender<McpRequest>>,
    /// Channel for receiving responses from server
    response_receiver: Option<mpsc::Receiver<McpResponse>>,
    /// Response sender for internal use
    response_sender: Option<mpsc::Sender<McpResponse>>,
    /// Flag to track if transport is running
    is_running: bool,
}

impl Transport {
    /// Create new transport instance
    pub fn new() -> Result<Self> {
        Ok(Self {
            request_sender: None,
            response_receiver: None,
            response_sender: None,
            is_running: false,
        })
    }

    /// Start the transport layer
    /// 
    /// Begins reading from STDIN for incoming requests and sets up response
    /// channel for outgoing messages. This function establishes the communication
    /// channels between the transport and the MCP server.
    #[instrument(skip(self, server_sender))]
    pub async fn start(&mut self, server_sender: mpsc::Sender<McpRequest>) -> Result<()> {
        info!("Starting STDIO transport layer");

        if self.is_running {
            return Err(anyhow!("Transport is already running"));
        }

        // Set up response channel
        let (response_tx, response_rx) = mpsc::channel::<McpResponse>(100);
        self.response_sender = Some(response_tx);
        self.response_receiver = Some(response_rx);
        self.request_sender = Some(server_sender);

        // Start STDIN reader task
        let request_sender = self.request_sender.as_ref().unwrap().clone();
        tokio::spawn(async move {
            if let Err(e) = Self::stdin_reader_task(request_sender).await {
                error!("STDIN reader task failed: {}", e);
            }
        });

        // Start STDOUT writer task
        let response_receiver = self.response_receiver.take().unwrap();
        tokio::spawn(async move {
            if let Err(e) = Self::stdout_writer_task(response_receiver).await {
                error!("STDOUT writer task failed: {}", e);
            }
        });

        self.is_running = true;
        info!("STDIO transport started successfully");
        
        Ok(())
    }

    /// Send response message to client via STDOUT
    #[instrument(skip(self, response))]
    pub async fn send_response(&self, response: McpResponse) -> Result<()> {
        if let Some(sender) = &self.response_sender {
            sender.send(response).await
                .map_err(|e| anyhow!("Failed to send response: {}", e))?;
            Ok(())
        } else {
            Err(anyhow!("Transport not started"))
        }
    }

    /// STDIN reader task - reads JSON-RPC messages from STDIN
    #[instrument(skip(request_sender))]
    async fn stdin_reader_task(request_sender: mpsc::Sender<McpRequest>) -> Result<()> {
        info!("Starting STDIN reader task");

        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line_buffer = String::new();

        loop {
            line_buffer.clear();
            
            match reader.read_line(&mut line_buffer).await {
                Ok(0) => {
                    info!("STDIN closed, stopping reader task");
                    break;
                }
                Ok(_) => {
                    let line = line_buffer.trim();
                    if line.is_empty() {
                        continue;
                    }

                    debug!("Received raw message: {}", line);

                    match Self::parse_request(line) {
                        Ok(request) => {
                            debug!("Parsed request: {:?}", request);
                            if let Err(e) = request_sender.send(request).await {
                                error!("Failed to forward request to server: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse request: {} - Raw: {}", e, line);
                            // Send error response for malformed requests
                            let error_response = McpResponse {
                                jsonrpc: "2.0".to_string(),
                                id: json!(null),
                                result: None,
                                error: Some(super::server::McpError {
                                    code: -32700, // Parse error
                                    message: format!("Parse error: {}", e),
                                    data: None,
                                }),
                            };
                            
                            if let Err(e) = Self::write_response_to_stdout(&error_response).await {
                                error!("Failed to send error response: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read from STDIN: {}", e);
                    break;
                }
            }
        }

        info!("STDIN reader task finished");
        Ok(())
    }

    /// STDOUT writer task - writes JSON-RPC responses to STDOUT
    #[instrument(skip(response_receiver))]
    async fn stdout_writer_task(mut response_receiver: mpsc::Receiver<McpResponse>) -> Result<()> {
        info!("Starting STDOUT writer task");

        while let Some(response) = response_receiver.recv().await {
            debug!("Sending response: {:?}", response);
            
            if let Err(e) = Self::write_response_to_stdout(&response).await {
                error!("Failed to write response to STDOUT: {}", e);
                // Continue processing other responses
            }
        }

        info!("STDOUT writer task finished");
        Ok(())
    }

    /// Parse incoming JSON-RPC request from string
    #[instrument(skip(line))]
    fn parse_request(line: &str) -> Result<McpRequest> {
        // Parse as generic JSON first
        let value: Value = serde_json::from_str(line)
            .map_err(|e| anyhow!("Invalid JSON: {}", e))?;

        // Validate JSON-RPC 2.0 structure
        if value["jsonrpc"] != "2.0" {
            return Err(anyhow!("Missing or invalid jsonrpc field"));
        }

        if value["method"].is_null() {
            return Err(anyhow!("Missing method field"));
        }

        // Parse as MCP request
        let request: McpRequest = serde_json::from_value(value)
            .map_err(|e| anyhow!("Invalid MCP request structure: {}", e))?;

        Ok(request)
    }

    /// Write response to STDOUT as JSON-RPC message
    #[instrument(skip(response))]
    async fn write_response_to_stdout(response: &McpResponse) -> Result<()> {
        let json_str = serde_json::to_string(response)
            .map_err(|e| anyhow!("Failed to serialize response: {}", e))?;

        let mut stdout = tokio::io::stdout();
        stdout.write_all(json_str.as_bytes()).await
            .map_err(|e| anyhow!("Failed to write to STDOUT: {}", e))?;
        stdout.write_all(b"\n").await
            .map_err(|e| anyhow!("Failed to write newline to STDOUT: {}", e))?;
        stdout.flush().await
            .map_err(|e| anyhow!("Failed to flush STDOUT: {}", e))?;

        debug!("Response written to STDOUT: {}", json_str);
        Ok(())
    }

    /// Stop the transport layer
    pub fn stop(&mut self) {
        info!("Stopping STDIO transport");
        self.is_running = false;
        self.request_sender = None;
        self.response_sender = None;
        self.response_receiver = None;
    }

    /// Check if transport is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

/// Helper functions for testing and debugging
impl Transport {
    /// Create a test message for validation
    #[cfg(test)]
    pub fn create_test_message(method: &str, params: Value) -> String {
        serde_json::to_string(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        })).unwrap()
    }

    /// Validate message format
    pub fn validate_message_format(message: &str) -> Result<()> {
        let value: Value = serde_json::from_str(message)?;
        
        if value["jsonrpc"] != "2.0" {
            return Err(anyhow!("Invalid jsonrpc version"));
        }

        if value["id"].is_null() && value["method"].is_null() {
            return Err(anyhow!("Message must have either id (response) or method (request)"));
        }

        Ok(())
    }

    /// Extract message type from raw message
    pub fn get_message_type(message: &str) -> Result<String> {
        let value: Value = serde_json::from_str(message)?;
        
        if let Some(method) = value["method"].as_str() {
            Ok(format!("request:{}", method))
        } else if !value["id"].is_null() {
            Ok("response".to_string())
        } else {
            Err(anyhow!("Unable to determine message type"))
        }
    }
}

/// Transport configuration options
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Buffer size for channels
    pub channel_buffer_size: usize,
    /// Enable debug logging of all messages
    pub debug_messages: bool,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Timeout for read operations in milliseconds
    pub read_timeout_ms: Option<u64>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            channel_buffer_size: 100,
            debug_messages: false,
            max_message_size: 1024 * 1024, // 1MB
            read_timeout_ms: None,
        }
    }
}

/// Transport statistics for monitoring
#[derive(Debug, Clone)]
pub struct TransportStats {
    pub messages_received: u64,
    pub messages_sent: u64,
    pub parse_errors: u64,
    pub write_errors: u64,
    pub start_time: chrono::DateTime<chrono::Utc>,
}

impl TransportStats {
    pub fn new() -> Self {
        Self {
            messages_received: 0,
            messages_sent: 0,
            parse_errors: 0,
            write_errors: 0,
            start_time: chrono::Utc::now(),
        }
    }

    pub fn uptime(&self) -> chrono::Duration {
        chrono::Utc::now() - self.start_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_valid_request() {
        let message = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","clientInfo":{"name":"test","version":"1.0"}}}"#;
        
        let result = Transport::parse_request(message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_json() {
        let message = r#"{"invalid json"#;
        
        let result = Transport::parse_request(message);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_jsonrpc() {
        let message = r#"{"id":1,"method":"test"}"#;
        
        let result = Transport::parse_request(message);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_method() {
        let message = r#"{"jsonrpc":"2.0","id":1}"#;
        
        let result = Transport::parse_request(message);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_message_format() {
        let valid_message = r#"{"jsonrpc":"2.0","id":1,"method":"test"}"#;
        assert!(Transport::validate_message_format(valid_message).is_ok());

        let invalid_message = r#"{"jsonrpc":"1.0","id":1}"#;
        assert!(Transport::validate_message_format(invalid_message).is_err());
    }

    #[test]
    fn test_get_message_type() {
        let request_message = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let message_type = Transport::get_message_type(request_message).unwrap();
        assert_eq!(message_type, "request:initialize");

        let response_message = r#"{"jsonrpc":"2.0","id":1,"result":{}}"#;
        let message_type = Transport::get_message_type(response_message).unwrap();
        assert_eq!(message_type, "response");
    }

    #[tokio::test]
    async fn test_transport_creation() {
        let transport = Transport::new().unwrap();
        assert!(!transport.is_running());
    }

    #[test]
    fn test_create_test_message() {
        let message = Transport::create_test_message("test", json!({}));
        let parsed: Value = serde_json::from_str(&message).unwrap();
        
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["method"], "test");
        assert_eq!(parsed["id"], 1);
    }

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert_eq!(config.channel_buffer_size, 100);
        assert!(!config.debug_messages);
        assert_eq!(config.max_message_size, 1024 * 1024);
        assert!(config.read_timeout_ms.is_none());
    }

    #[test]
    fn test_transport_stats() {
        let stats = TransportStats::new();
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.parse_errors, 0);
        assert_eq!(stats.write_errors, 0);
        assert!(stats.uptime().num_seconds() >= 0);
    }
}