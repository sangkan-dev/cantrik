//! Heuristic task complexity for smart routing (Sprint 14).

use super::providers::{
    ProviderTarget, ProvidersLoadError, ProvidersToml, RoutingThresholds, route_entry_to_target,
};

/// Coarse tier used to pick a route from `[routing.thresholds]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskTier {
    Simple,
    Medium,
    Complex,
}

impl TaskTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Medium => "medium",
            Self::Complex => "complex",
        }
    }
}

/// Deterministic tier from prompt text (length + keywords).
pub fn classify_prompt(text: &str) -> TaskTier {
    let t = text.trim();
    let lower = t.to_ascii_lowercase();

    let complex_kw = [
        "audit",
        "security",
        "refactor",
        "architecture",
        "thread",
        "race",
        "deadlock",
        "cryptograph",
        "compliance",
    ];
    let medium_kw = [
        "explain", "debug", "test", "why", "how does", "trace", "profile",
    ];

    if complex_kw.iter().any(|k| lower.contains(k)) {
        return TaskTier::Complex;
    }
    if medium_kw.iter().any(|k| lower.contains(k)) {
        return TaskTier::Medium;
    }

    let word_count = lower.split_whitespace().count();
    let len = t.chars().count();
    if len > 4_000 || word_count > 600 {
        TaskTier::Complex
    } else if len > 1_200 || word_count > 180 {
        TaskTier::Medium
    } else {
        TaskTier::Simple
    }
}

fn threshold_route(tier: TaskTier, th: &RoutingThresholds) -> Option<&str> {
    let s = match tier {
        TaskTier::Simple => th.simple.as_deref(),
        TaskTier::Medium => th.medium.as_deref(),
        TaskTier::Complex => th.complex.as_deref(),
    };
    s.map(str::trim).filter(|x| !x.is_empty())
}

/// Resolve `provider/model` (or `provider`) for this tier using `providers.toml` defaults.
pub fn resolve_routed_target(
    tier: TaskTier,
    thresholds: &RoutingThresholds,
    providers: &ProvidersToml,
) -> Result<Option<ProviderTarget>, ProvidersLoadError> {
    let Some(entry) = threshold_route(tier, thresholds) else {
        return Ok(None);
    };
    route_entry_to_target(entry, providers).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_keywords() {
        assert_eq!(
            classify_prompt("Please audit the auth flow for security issues"),
            TaskTier::Complex
        );
        assert_eq!(
            classify_prompt("Explain how this function works"),
            TaskTier::Medium
        );
        assert_eq!(classify_prompt("hi"), TaskTier::Simple);
    }

    #[test]
    fn tier_length() {
        let long = "word ".repeat(200);
        assert_eq!(classify_prompt(&long), TaskTier::Medium);
        let very_long = "x".repeat(5000);
        assert_eq!(classify_prompt(&very_long), TaskTier::Complex);
    }
}
