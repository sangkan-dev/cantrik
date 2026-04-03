//! Read-only `git` subcommand allowlist (Sprint 8).

// `remote` / `config` / `stash` intentionally omitted (can trigger network or writes with extra args).
const ALLOW: &[&str] = &[
    "status",
    "diff",
    "log",
    "show",
    "rev-parse",
    "branch",
    "describe",
    "tag",
    "shortlog",
    "whatchanged",
    "ls-files",
    "ls-tree",
    "cat-file",
    "check-ignore",
    "blame",
    "grep",
    "symbolic-ref",
    "name-rev",
];

pub fn check_git_args(args: &[String]) -> Result<(), &'static str> {
    let Some(first) = args.first().map(|s| s.as_str()) else {
        return Err("git: missing subcommand");
    };
    if ALLOW.contains(&first) {
        Ok(())
    } else {
        Err("git: subcommand not in read-only allowlist")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_log() {
        assert!(check_git_args(&["log".into(), "-1".into()]).is_ok());
    }

    #[test]
    fn blocks_push() {
        assert!(check_git_args(&["push".into()]).is_err());
    }
}
