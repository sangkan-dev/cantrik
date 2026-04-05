use std::io::Write;
use std::process::ExitCode;

use cantrik_core::config::{AppConfig, offline_blocks_outbound_http, offline_http_blocked_message};
use cantrik_core::tool_system::{NetworkApproval, tool_web_fetch};

pub(crate) async fn run(config: &AppConfig, url: &str, approve: bool, max_bytes: u64) -> ExitCode {
    if !approve {
        eprintln!("fetch: would GET {url} (cap {max_bytes} bytes); use --approve.");
        return ExitCode::SUCCESS;
    }
    if offline_blocks_outbound_http(config) {
        eprintln!("fetch: {}", offline_http_blocked_message());
        return ExitCode::from(2);
    }

    match tool_web_fetch(
        config,
        url,
        max_bytes,
        NetworkApproval::user_approved_network(),
    )
    .await
    {
        Ok(body) => match std::io::stdout().write_all(&body) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("fetch: {e}");
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("fetch: {e}");
            ExitCode::FAILURE
        }
    }
}
