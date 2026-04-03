use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    cantrik_cli::run().await
}
