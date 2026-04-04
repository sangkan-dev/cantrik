use std::process::ExitCode;

pub fn run(issue_url: &str) -> ExitCode {
    let u = issue_url.trim();
    if u.is_empty() {
        eprintln!("fix: issue URL required");
        return ExitCode::from(2);
    }
    if !(u.starts_with("http://") || u.starts_with("https://")) {
        eprintln!("fix: expected http(s) URL");
        return ExitCode::from(2);
    }
    println!("Issue: {u}");
    println!("MVP: full SWE-agent loop deferred. Try:");
    println!("  cantrik fetch {u} --approve");
    println!("  cantrik agents \"Address issue {u}\"");
    println!("  cantrik workspace commit --approve && cantrik pr create --approve");
    ExitCode::SUCCESS
}
