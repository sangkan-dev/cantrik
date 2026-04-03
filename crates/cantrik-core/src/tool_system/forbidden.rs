//! Hard blocks for obviously dangerous invocation patterns (PRD §5).

/// Returns an error message if argv should never run.
pub fn check_exec_argv(argv: &[String]) -> Result<(), &'static str> {
    if argv.is_empty() {
        return Err("empty argv");
    }
    let joined = argv.join(" ").to_lowercase();
    // Minimal guardrails; expand in later sprints.
    if joined.contains("rm ") && (joined.contains("-rf") || joined.contains("-fr")) {
        return Err("refused: rm -rf pattern");
    }
    if joined.contains("mkfs.") || joined.contains(" dd ") {
        return Err("refused: destructive disk command pattern");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_rm_rf() {
        let v = vec!["rm".into(), "-rf".into(), "/".into()];
        assert!(check_exec_argv(&v).is_err());
    }

    #[test]
    fn allows_echo() {
        let v = vec!["echo".into(), "hi".into()];
        assert!(check_exec_argv(&v).is_ok());
    }
}
