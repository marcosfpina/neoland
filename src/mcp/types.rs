use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: &'static str, params: Option<Value>) -> Self {
        Self { jsonrpc: "2.0", id, method, params }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse {
    pub id: Option<u64>,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[derive(Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
}

pub struct CallToolRequest {
    pub name: String,
    pub arguments: Value,
}

#[derive(Serialize)]
pub struct CallToolResult {
    pub text: String,
    pub is_error: bool,
}
