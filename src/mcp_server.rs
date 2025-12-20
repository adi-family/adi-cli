//! MCP (Model Context Protocol) server that dispatches to plugin-provided tools and resources.

use std::io::{self, BufRead, Write};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::plugin_runtime::PluginRuntime;

/// MCP server that reads JSON-RPC from stdin and writes responses to stdout.
pub struct McpServer {
    runtime: Arc<PluginRuntime>,
    initialized: bool,
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl McpServer {
    /// Create a new MCP server with the given plugin runtime.
    pub fn new(runtime: Arc<PluginRuntime>) -> Self {
        Self {
            runtime,
            initialized: false,
        }
    }

    /// Run the MCP server, reading from stdin and writing to stdout.
    pub async fn run(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            let response = self.handle_request(&line).await;
            let response_json = serde_json::to_string(&response)?;
            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }

        Ok(())
    }

    async fn handle_request(&mut self, line: &str) -> JsonRpcResponse {
        let request: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(req) => req,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
            }
        };

        let id = request.id.clone().unwrap_or(Value::Null);

        match self.dispatch(&request).await {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(result),
                error: None,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(e),
            },
        }
    }

    async fn dispatch(&mut self, request: &JsonRpcRequest) -> Result<Value, JsonRpcError> {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(&request.params),
            "initialized" => Ok(Value::Null),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(&request.params),
            "resources/list" => self.handle_resources_list(),
            "resources/read" => self.handle_resources_read(&request.params),
            "prompts/list" => Ok(json!({ "prompts": [] })),
            "prompts/get" => Err(JsonRpcError {
                code: -32601,
                message: "Prompt not found".to_string(),
                data: None,
            }),
            "ping" => Ok(json!({})),
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        }
    }

    fn handle_initialize(&mut self, params: &Value) -> Result<Value, JsonRpcError> {
        self.initialized = true;

        // Get protocol version from params
        let protocol_version = params
            .get("protocolVersion")
            .and_then(|v| v.as_str())
            .unwrap_or("2024-11-05");

        Ok(json!({
            "protocolVersion": protocol_version,
            "capabilities": {
                "tools": { "listChanged": false },
                "resources": { "listChanged": false, "subscribe": false },
                "prompts": { "listChanged": false }
            },
            "serverInfo": {
                "name": "adi-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        }))
    }

    fn handle_tools_list(&self) -> Result<Value, JsonRpcError> {
        // Try to get tools from plugins
        match self.runtime.list_mcp_tools() {
            Ok(tools_json) => {
                let tools: Value = serde_json::from_str(&tools_json).unwrap_or(json!([]));
                Ok(json!({ "tools": tools }))
            }
            Err(_) => {
                // No plugins provide tools, return empty list
                Ok(json!({ "tools": [] }))
            }
        }
    }

    fn handle_tools_call(&self, params: &Value) -> Result<Value, JsonRpcError> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing 'name' parameter".to_string(),
                data: None,
            })?;

        let default_args = json!({});
        let args = params.get("arguments").unwrap_or(&default_args);
        let args_str = serde_json::to_string(args).unwrap_or_else(|_| "{}".to_string());

        match self.runtime.call_mcp_tool(name, &args_str) {
            Ok(result_json) => {
                let result: Value =
                    serde_json::from_str(&result_json).unwrap_or(json!({ "content": result_json }));
                Ok(result)
            }
            Err(e) => Err(JsonRpcError {
                code: -32000,
                message: format!("Tool call failed: {}", e),
                data: None,
            }),
        }
    }

    fn handle_resources_list(&self) -> Result<Value, JsonRpcError> {
        match self.runtime.list_mcp_resources() {
            Ok(resources_json) => {
                let resources: Value = serde_json::from_str(&resources_json).unwrap_or(json!([]));
                Ok(json!({ "resources": resources }))
            }
            Err(_) => Ok(json!({ "resources": [] })),
        }
    }

    fn handle_resources_read(&self, params: &Value) -> Result<Value, JsonRpcError> {
        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing 'uri' parameter".to_string(),
                data: None,
            })?;

        match self.runtime.read_mcp_resource(uri) {
            Ok(content_json) => {
                let content: Value = serde_json::from_str(&content_json)
                    .unwrap_or(json!({ "uri": uri, "text": content_json }));
                Ok(json!({ "contents": [content] }))
            }
            Err(e) => Err(JsonRpcError {
                code: -32000,
                message: format!("Resource read failed: {}", e),
                data: None,
            }),
        }
    }
}
