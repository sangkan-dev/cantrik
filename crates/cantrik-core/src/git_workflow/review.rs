pub const REVIEW_SYSTEM_PROMPT: &str = r#"You are a senior code reviewer. Analyze the unified diff below.

Checklist:
1) Security: injection, secrets, unsafe patterns, auth gaps.
2) Correctness: logic errors, error handling, edge cases.
3) Performance: obvious hot-path issues.
4) Style & maintainability: naming, duplication, test gaps.

Output concise bullet findings (max ~40 lines). If nothing stands out, say "No major issues spotted" and optional nits."#;

/// Build user message containing diff context.
pub fn build_review_prompt(diff: &str, staged: bool) -> String {
    let label = if staged {
        "staged (index) diff"
    } else {
        "working tree vs HEAD"
    };
    format!(
        "{REVIEW_SYSTEM_PROMPT}\n\n## {label}\n\n```diff\n{diff}\n```\n",
        REVIEW_SYSTEM_PROMPT = REVIEW_SYSTEM_PROMPT,
        label = label,
        diff = diff
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_prompt_contains_diff() {
        let p = build_review_prompt("+fn x() {}", true);
        assert!(p.contains("+fn x()"));
        assert!(p.contains("staged"));
    }
}
