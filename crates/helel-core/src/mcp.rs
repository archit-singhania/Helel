//! Local MCP JSON-RPC contract. Transport policy permits local stdio only.

use crate::local_process::JsonLineProcess;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::HashMap, fs, io, path::Path, time::Duration};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpServerConfig {
    pub name: String,
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "enabled")]
    pub enabled: bool,
}
const fn enabled() -> bool {
    true
}

/// Saves local stdio MCP server definitions inside the workspace.
///
/// # Errors
/// Returns an error when configuration serialization or atomic persistence fails.
pub fn save_registry(root: &Path, servers: &[McpServerConfig]) -> io::Result<()> {
    let directory = root.join(".helel");
    fs::create_dir_all(&directory)?;
    let temporary = directory.join("mcp.json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(servers).map_err(io::Error::other)?,
    )?;
    fs::rename(temporary, directory.join("mcp.json"))
}

/// Loads local MCP definitions, returning an empty registry when absent.
///
/// # Errors
/// Returns an error for malformed configuration.
pub fn load_registry(root: &Path) -> io::Result<Vec<McpServerConfig>> {
    let path = root.join(".helel/mcp.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    serde_json::from_slice(&fs::read(path)?).map_err(io::Error::other)
}

pub struct McpClient {
    process: JsonLineProcess,
    next_id: u64,
    pending: HashMap<u64, String>,
}
impl McpClient {
    /// Starts one configured local stdio MCP server.
    ///
    /// # Errors
    /// Returns an error when the direct child process cannot start.
    pub fn start(config: &McpServerConfig) -> io::Result<Self> {
        if !config.enabled {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "MCP server is disabled",
            ));
        }
        Ok(Self {
            process: JsonLineProcess::start(&config.program, &config.args)?,
            next_id: 1,
            pending: HashMap::new(),
        })
    }
    /// Sends one MCP request and validates its matching response.
    ///
    /// # Errors
    /// Returns an error for transport failures, malformed responses, mismatched IDs, or server errors.
    pub fn call(&mut self, method: &str, params: &Value) -> io::Result<Value> {
        if method.is_empty() || method.len() > 256 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid MCP method",
            ));
        }
        let id = self.next_id;
        self.next_id += 1;
        self.pending.insert(id, method.into());
        let encoded = json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}).to_string();
        let line = self.process.request(&encoded, Duration::from_secs(30))?;
        let response: JsonRpcResponse = serde_json::from_str(&line).map_err(io::Error::other)?;
        if response.jsonrpc != "2.0" || response.id != id || self.pending.remove(&id).is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "mismatched MCP response",
            ));
        }
        if let Some(error) = response.error {
            return Err(io::Error::other(format!(
                "MCP {}: {}",
                error.code, error.message
            )));
        }
        response
            .result
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "MCP response has no result"))
    }
    /// Performs MCP 2025-06-18 capability negotiation and sends the initialized notification.
    ///
    /// # Errors
    /// Returns an error for transport failure or a mismatched negotiated version.
    pub fn initialize(&mut self) -> io::Result<Value> {
        let result=self.call("initialize",&json!({"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"Helel","version":"0.1.0"}}))?;
        if result.get("protocolVersion").and_then(Value::as_str) != Some("2025-06-18") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "MCP protocol version was not negotiated",
            ));
        }
        self.process
            .send(&json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string())?;
        Ok(result)
    }
    /// Discovers tools after initialization.
    ///
    /// # Errors
    /// Returns protocol or transport errors.
    pub fn list_tools(&mut self) -> io::Result<Value> {
        self.call("tools/list", &json!({}))
    }
    /// Calls one discovered tool with explicit arguments.
    ///
    /// # Errors
    /// Returns protocol or transport errors.
    pub fn call_tool(&mut self, name: &str, arguments: &Value) -> io::Result<Value> {
        if name.is_empty() || name.len() > 256 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid MCP tool name",
            ));
        }
        self.call("tools/call", &json!({"name":name,"arguments":arguments}))
    }
    /// Stops the MCP server.
    ///
    /// # Errors
    /// Returns an error when termination fails.
    pub fn stop(&self) -> io::Result<()> {
        self.process.stop()
    }
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
    #[test]
    fn registry_round_trips() {
        let d = tempfile::tempdir().unwrap();
        let servers = vec![McpServerConfig {
            name: "fixture".into(),
            program: "fixture-mcp".into(),
            args: vec!["--stdio".into()],
            enabled: true,
        }];
        save_registry(d.path(), &servers).unwrap();
        assert_eq!(load_registry(d.path()).unwrap(), servers);
    }
    #[test]
    fn initializes_and_discovers_stdio_tools() {
        let code = "import sys,json\nfor line in sys.stdin:\n p=json.loads(line)\n if 'id' not in p: continue\n if p['method']=='initialize': result={'protocolVersion':'2025-06-18','capabilities':{'tools':{}}}\n elif p['method']=='tools/list': result={'tools':[{'name':'fixture','inputSchema':{'type':'object'}}]}\n else: result={'content':[{'type':'text','text':str(p['params']['arguments']['value'])}]}\n print(json.dumps({'jsonrpc':'2.0','id':p['id'],'result':result}),flush=True)";
        let config = McpServerConfig {
            name: "fixture".into(),
            program: "python3".into(),
            args: vec!["-u".into(), "-c".into(), code.into()],
            enabled: true,
        };
        let mut client = McpClient::start(&config).unwrap();
        assert_eq!(
            client.initialize().unwrap()["protocolVersion"],
            "2025-06-18"
        );
        assert_eq!(client.list_tools().unwrap()["tools"][0]["name"], "fixture");
        assert_eq!(
            client.call_tool("fixture", &json!({"value": 42})).unwrap()["content"][0]["text"],
            "42"
        );
        client.stop().unwrap();
    }
}
