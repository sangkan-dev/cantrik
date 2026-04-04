//! LSP server over stdio (Sprint 18): symbols from `.cantrik/index/ast/chunks.jsonl`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cantrik_core::indexing::{load_chunks_for_document, outlines_from_chunks};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    DocumentSymbolParams, DocumentSymbolResponse, Hover, HoverContents, HoverParams,
    InitializeParams, InitializeResult, Location, MarkedString, MessageType, OneOf, Position,
    Range, ServerCapabilities, ServerInfo, SymbolInformation, SymbolKind,
    TextDocumentSyncCapability, TextDocumentSyncKind,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

static LOGGED_MISSING_CHUNKS: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
struct Backend {
    client: Client,
    root: Arc<RwLock<Option<PathBuf>>>,
}

fn resolve_workspace_root(params: &InitializeParams) -> Option<PathBuf> {
    if let Some(folders) = &params.workspace_folders {
        if let Some(first) = folders.first() {
            return first.uri.to_file_path().ok();
        }
    }
    params.root_uri.as_ref().and_then(|u| u.to_file_path().ok())
}

fn map_chunk_kind(s: &str) -> SymbolKind {
    match s.to_ascii_lowercase().as_str() {
        "function" | "fn" => SymbolKind::FUNCTION,
        "struct" => SymbolKind::STRUCT,
        "enum" => SymbolKind::ENUM,
        "trait" => SymbolKind::INTERFACE,
        "impl" => SymbolKind::OBJECT,
        "module" | "mod" => SymbolKind::MODULE,
        "field" | "property" => SymbolKind::FIELD,
        "method" => SymbolKind::METHOD,
        "class" => SymbolKind::CLASS,
        "variable" | "let" | "const" => SymbolKind::VARIABLE,
        "interface" => SymbolKind::INTERFACE,
        _ => SymbolKind::OBJECT,
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let root = resolve_workspace_root(&params);
        *self.root.write().await = root;
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::NONE,
                )),
                document_symbol_provider: Some(OneOf::Left(true)),
                hover_provider: Some(tower_lsp::lsp_types::HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "cantrik".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            ..Default::default()
        })
    }

    async fn initialized(&self, _: tower_lsp::lsp_types::InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                "Cantrik LSP ready (index with `cantrik index`).",
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    #[allow(deprecated)]
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let path = match uri.to_file_path() {
            Ok(p) => p,
            Err(()) => return Ok(Some(Vec::<SymbolInformation>::new().into())),
        };
        let proj = self.root.read().await.clone();
        let Some(proj_root) = proj else {
            return Ok(Some(Vec::<SymbolInformation>::new().into()));
        };
        let ast_dir = proj_root.join(".cantrik").join("index").join("ast");
        let chunks_file = ast_dir.join("chunks.jsonl");
        if !chunks_file.exists() {
            if !LOGGED_MISSING_CHUNKS.swap(true, Ordering::SeqCst) {
                eprintln!(
                    "cantrik lsp: no index at {}; run `cantrik index` in the workspace root.",
                    chunks_file.display()
                );
            }
            return Ok(Some(Vec::<SymbolInformation>::new().into()));
        }
        let chunks = match load_chunks_for_document(&proj_root, &path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("cantrik lsp: failed to read chunks: {e}");
                return Ok(Some(Vec::<SymbolInformation>::new().into()));
            }
        };
        let outlines = outlines_from_chunks(&chunks);
        let mut syms = Vec::with_capacity(outlines.len());
        for o in &outlines {
            syms.push(SymbolInformation {
                name: o.name.clone(),
                kind: map_chunk_kind(&o.kind),
                tags: None,
                deprecated: None,
                location: Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position {
                            line: o.start_row,
                            character: o.start_col,
                        },
                        end: Position {
                            line: o.end_row,
                            character: o.end_col,
                        },
                    },
                },
                container_name: None,
            });
        }
        Ok(Some(syms.into()))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let path = match uri.to_file_path() {
            Ok(p) => p,
            Err(()) => return Ok(None),
        };
        let proj = self.root.read().await.clone();
        let Some(proj_root) = proj else {
            return Ok(None);
        };
        let chunks = load_chunks_for_document(&proj_root, &path).unwrap_or_default();
        let outlines = outlines_from_chunks(&chunks);
        for (c, o) in chunks.iter().zip(outlines.iter()) {
            let in_range = pos.line >= o.start_row
                && pos.line <= o.end_row
                && (pos.line != o.start_row || pos.character >= o.start_col)
                && (pos.line != o.end_row || pos.character <= o.end_col);
            if in_range {
                let head = c.source.lines().next().unwrap_or_default();
                let text = format!(
                    "`{}` ({kind})\n\n{head}\n\n_Indexed by Cantrik — run `cantrik index` to refresh._",
                    c.symbol,
                    kind = c.kind,
                    head = head
                );
                return Ok(Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(text)),
                    range: None,
                }));
            }
        }
        Ok(None)
    }
}

/// Run the LSP server on stdio until the client disconnects.
pub async fn run() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| Backend {
        client,
        root: Arc::new(RwLock::new(None)),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}
