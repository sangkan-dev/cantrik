//! MCP stdio server: `cantrik serve --mcp` (Sprint 14).

use std::process::ExitCode;
use std::sync::Arc;

use cantrik_core::config::AppConfig;
use cantrik_core::llm;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, JsonObject, ListToolsResult, PaginatedRequestParams,
    ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler, ServiceExt};

#[derive(Clone)]
struct CantrikMcpService {
    config: AppConfig,
}

impl ServerHandler for CantrikMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let schema: JsonObject = serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Prompt sent to the configured Cantrik LLM (non-streaming)."
                }
            },
            "required": ["prompt"]
        })
        .as_object()
        .expect("tool schema object")
        .clone();

        let tool = Tool::new(
            "cantrik_ask",
            "Run one Cantrik LLM completion and return the full assistant text.",
            Arc::new(schema),
        );

        Ok(ListToolsResult {
            tools: vec![tool],
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        if request.name.as_ref() != "cantrik_ask" {
            return Ok(CallToolResult::structured_error(serde_json::json!({
                "error": format!("unknown tool {:?}", request.name)
            })));
        }
        let Some(args) = request.arguments.as_ref() else {
            return Ok(CallToolResult::structured_error(serde_json::json!({
                "error": "missing tool arguments object"
            })));
        };
        let Some(p) = args.get("prompt") else {
            return Ok(CallToolResult::structured_error(serde_json::json!({
                "error": "missing string field \"prompt\""
            })));
        };
        let Some(prompt) = p.as_str() else {
            return Ok(CallToolResult::structured_error(serde_json::json!({
                "error": "field \"prompt\" must be a string"
            })));
        };
        if prompt.trim().is_empty() {
            return Ok(CallToolResult::structured_error(serde_json::json!({
                "error": "prompt is empty"
            })));
        }

        match llm::ask_complete_text(&self.config, prompt).await {
            Ok(text) => Ok(CallToolResult::structured(
                serde_json::json!({ "reply": text }),
            )),
            Err(e) => Ok(CallToolResult::structured_error(serde_json::json!({
                "error": e.to_string()
            }))),
        }
    }
}

pub async fn run_mcp_stdio(config: AppConfig) -> ExitCode {
    let service = CantrikMcpService { config };
    let transport = (tokio::io::stdin(), tokio::io::stdout());
    let running = match service.serve(transport).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("mcp serve: handshake failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = running.waiting().await {
        eprintln!("mcp serve: service ended with error: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
