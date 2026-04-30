pub mod client;
pub mod registry;
pub mod types;
pub mod server;

pub use client::McpClient;
pub use registry::McpRegistry;
pub use types::{CallToolResult, Tool};

#[cfg(test)]
mod tests {
    use super::types::*;

    #[test]
    fn deserialize_tool() {
        let json = r#"{"name":"search_knowledge","description":"Search the knowledge base"}"#;
        let tool: Tool = serde_json::from_str(json).unwrap();
        assert_eq!(tool.name, "search_knowledge");
        assert_eq!(tool.description, "Search the knowledge base");
    }

    #[test]
    fn deserialize_rpc_response_ok() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"tools":[]}}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, Some(1));
        assert!(resp.error.is_none());
        assert!(resp.result.is_some());
    }

    #[test]
    fn deserialize_rpc_response_error() {
        let json =
            r#"{"jsonrpc":"2.0","id":2,"error":{"code":-32601,"message":"Method not found"}}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, Some(2));
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32601);
        assert_eq!(err.message, "Method not found");
    }

    #[test]
    fn deserialize_rpc_notification_has_no_id() {
        let json = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(resp.id.is_none());
        assert!(resp.error.is_none());
    }

    #[test]
    fn call_tool_result_ok() {
        let r = CallToolResult { text: "done".to_string(), is_error: false };
        assert!(!r.is_error);
    }

    #[test]
    fn call_tool_result_error() {
        let r = CallToolResult { text: "failed".to_string(), is_error: true };
        assert!(r.is_error);
    }

    #[test]
    fn serialize_rpc_request() {
        let req = JsonRpcRequest::new(3, "tools/list", None);
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""id":3"#));
        assert!(json.contains(r#""method":"tools/list""#));
        assert!(!json.contains("params")); // skipped when None
    }
}
