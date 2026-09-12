//! Local MCP JSON-RPC contract. Transport policy permits local stdio only.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

/// Validates one bounded JSON-RPC request.
///
/// # Errors
/// Returns a description when the message is malformed or outside limits.
pub fn validate_request(line: &str) -> Result<JsonRpcRequest, String> {
    if line.len() > 1_048_576 {
        return Err("MCP message is oversized".into());
    }
    let request: JsonRpcRequest =
        serde_json::from_str(line).map_err(|e| format!("invalid MCP message: {e}"))?;
    if request.jsonrpc != "2.0" || request.method.is_empty() || request.method.len() > 256 {
        return Err("invalid MCP request".into());
    }
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_local_protocol_messages() {
        assert!(validate_request(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#).is_ok());
        assert!(validate_request(r#"{"jsonrpc":"1.0","id":1,"method":"x"}"#).is_err());
    }
}
