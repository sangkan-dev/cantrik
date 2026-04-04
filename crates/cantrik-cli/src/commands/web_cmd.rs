use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use cantrik_core::config::{
    effective_web_fetch_max_bytes, effective_web_search_max_results, load_merged_config,
};
use cantrik_core::tool_system::{NetworkApproval, tool_web_fetch, tool_web_search};

#[derive(Debug, Clone, clap::Subcommand)]
pub enum WebCommand {
    /// Search the web (DuckDuckGo HTML; best-effort parsing).
    Search {
        #[arg(required = true, trailing_var_arg = true, value_name = "QUERY")]
        query: Vec<String>,
        #[arg(long)]
        approve: bool,
        #[arg(long, default_value_t = 5usize)]
        max_results: usize,
        #[arg(long, default_value_t = 2_000_000_u64)]
        max_response_bytes: u64,
    },
    /// GET a URL (same as `cantrik fetch`).
    Fetch {
        url: String,
        #[arg(long)]
        approve: bool,
        #[arg(long, default_value_t = 2_000_000_u64)]
        max_bytes: u64,
    },
}

pub async fn run(cwd: &Path, sub: &WebCommand) -> ExitCode {
    let config = match load_merged_config(cwd) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to load config: {e}");
            return ExitCode::FAILURE;
        }
    };

    match sub {
        WebCommand::Search {
            query,
            approve,
            max_results,
            max_response_bytes,
        } => {
            if !approve {
                eprintln!("web search: dry run only; re-run with --approve to query the network.");
                println!("query: {}", query.join(" "));
                return ExitCode::SUCCESS;
            }
            let q = query.join(" ");
            let cap_results =
                effective_web_search_max_results(&config.git_workflow).min(*max_results);
            match tool_web_search(
                &config,
                &q,
                cap_results,
                *max_response_bytes,
                NetworkApproval::user_approved_network(),
            )
            .await
            {
                Ok(s) => {
                    print!("{s}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("web search: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        WebCommand::Fetch {
            url,
            approve,
            max_bytes,
        } => {
            if !approve {
                eprintln!("web fetch: would GET {url}; use --approve (same as `cantrik fetch`).");
                return ExitCode::SUCCESS;
            }
            let cap = effective_web_fetch_max_bytes(&config.git_workflow, *max_bytes);
            match tool_web_fetch(&config, url, cap, NetworkApproval::user_approved_network()).await
            {
                Ok(body) => match std::io::stdout().write_all(&body) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("web fetch: {e}");
                        ExitCode::FAILURE
                    }
                },
                Err(e) => {
                    eprintln!("web fetch: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
