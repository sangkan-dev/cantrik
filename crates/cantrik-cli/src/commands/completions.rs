use std::io;
use std::process::ExitCode;

use clap::CommandFactory;

use crate::cli::{Cli, CompletionShell};

pub(crate) fn run(shell: CompletionShell) -> ExitCode {
    let mut cmd = Cli::command();
    let shell: clap_complete::Shell = shell.into();
    clap_complete::generate(shell, &mut cmd, "cantrik", &mut io::stdout());
    ExitCode::SUCCESS
}
