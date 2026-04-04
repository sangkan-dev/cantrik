//! MCP stdio client: `cantrik mcp call` (Sprint 14).

use std::path::Path;
use std::process::ExitCode;

use cantrik_core::llm::providers::{McpServerEntry, load_providers_toml, providers_toml_path};
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use rmcp::transport::child_process::TokioChildProcess;

#[derive(Debug)]
enum McpClientCmdError {
    Providers(cantrik_core::llm::providers::ProvidersLoadError),
    Server(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    Rmcp(String),
}

impl std::fmt::Display for McpClientCmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpClientCmdError::Providers(e) => write!(f, "{e}"),
            McpClientCmdError::Server(s) => write!(f, "{s}"),
            McpClientCmdError::Io(e) => write!(f, "{e}"),
            McpClientCmdError::Json(e) => write!(f, "{e}"),
            McpClientCmdError::Rmcp(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for McpClientCmdError {}

impl From<cantrik_core::llm::providers::ProvidersLoadError> for McpClientCmdError {
    fn from(e: cantrik_core::llm::providers::ProvidersLoadError) -> Self {
        Self::Providers(e)
    }
}

fn find_server<'a>(
    servers: &'a [McpServerEntry],
    name: &str,
) -> Result<&'a McpServerEntry, McpClientCmdError> {
    servers.iter().find(|s| s.name == name).ok_or_else(|| {
        McpClientCmdError::Server(format!("no [[mcp_client.servers]] named {name:?}"))
    })
}

pub async fn call_tool(
    _cwd: &Path,
    server_name: &str,
    tool_name: &str,
    json_args: &str,
) -> Result<(), McpClientCmdError> {
    let path = providers_toml_path();
    let prov = load_providers_toml(&path)?;
    let section = prov.mcp_client.as_ref().ok_or_else(|| {
        McpClientCmdError::Server("missing [mcp_client] in providers.toml".into())
    })?;
    let entry = find_server(&section.servers, server_name)?;

    let mut cmd = tokio::process::Command::new(&entry.command);
    cmd.args(&entry.args);
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::inherit());

    let transport = TokioChildProcess::new(cmd).map_err(McpClientCmdError::Io)?;
    let client = ().serve(transport).await.map_err(|e| McpClientCmdError::Rmcp(e.to_string()))?;

    let v: serde_json::Value = serde_json::from_str(json_args).map_err(McpClientCmdError::Json)?;
    let obj = v
        .as_object()
        .cloned()
        .ok_or_else(|| McpClientCmdError::Server("JSON args must be a JSON object".into()))?;

    let params = CallToolRequestParams::new(tool_name.to_string()).with_arguments(obj);
    let out = client
        .call_tool(params)
        .await
        .map_err(|e| McpClientCmdError::Rmcp(e.to_string()))?;

    println!(
        "{}",
        serde_json::to_string_pretty(&out).unwrap_or_else(|_| format!("{out:?}"))
    );
    let _ = client.cancel().await;
    Ok(())
}

pub async fn run_call(
    cwd: &Path,
    server_name: String,
    tool_name: String,
    json_args: String,
) -> ExitCode {
    match call_tool(cwd, &server_name, &tool_name, &json_args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mcp call: {e}");
            ExitCode::FAILURE
        }
    }
}
