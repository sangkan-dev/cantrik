//! Deterministic risk heuristics (Sprint 15 MVP, no LLM required).

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl RiskLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

const SENSITIVE: &[&str] = &[
    "unsafe",
    "password",
    "secret",
    "token",
    "credential",
    "auth",
    "crypto",
    "encrypt",
    "decrypt",
    "private_key",
];

/// Score risk from changed file paths and raw diff+new text snippets.
pub fn assess_risk(
    paths: &[String],
    combined_diff_text: &str,
    total_files: usize,
    approx_loc_added: usize,
) -> (RiskLevel, Vec<String>) {
    let mut notes = Vec::new();
    let mut level = RiskLevel::Low;

    let lower = combined_diff_text.to_ascii_lowercase();
    for kw in SENSITIVE {
        if lower.contains(kw) {
            level = RiskLevel::High;
            notes.push(format!("diff mentions sensitive keyword `{kw}`"));
        }
    }

    let mut all_tests = true;
    for p in paths {
        let pl = p.to_ascii_lowercase();
        if pl.contains("/tests/")
            || pl.contains("\\tests\\")
            || pl.ends_with("_test.rs")
            || pl.contains("test.rs")
        {
            continue;
        }
        all_tests = false;
        break;
    }
    if !paths.is_empty() && all_tests {
        notes.push("only test paths changed — typical low impact".into());
        if level == RiskLevel::Low {
            notes.push("heuristic: prefer running targeted tests before merge".into());
        }
    }

    if total_files > 12 {
        level = level.max(RiskLevel::Medium);
        notes.push(format!("many files touched ({total_files}) — review scope"));
    }
    if approx_loc_added > 400 {
        level = level.max(RiskLevel::Medium);
        notes.push(format!(
            "large churn (~{approx_loc_added} added lines in diff)"
        ));
    }

    (level, notes)
}
