//! Optional prompt tone (PRD §6 Enhancement, Sprint 18).

use crate::config::CulturalWisdomLevel;

/// Short block injected into assembled LLM prompts when not `Off`.
pub fn prompt_addon(level: CulturalWisdomLevel) -> Option<String> {
    match level {
        CulturalWisdomLevel::Off => None,
        CulturalWisdomLevel::Light => Some(
            "Style note (cultural wisdom, light): Prefer clear, proportional changes — work with patience and care. Balance security, readability, and scope; avoid unnecessary complexity.\n"
                .to_string(),
        ),
        CulturalWisdomLevel::Full => Some(
            "Style note (cultural wisdom, full): Where it fits, you may add a very brief Indonesian gloss tying a Javanese collaborative or humility concept (e.g. working together calmly, humble review) to concrete engineering advice. One short clause maximum; keep the answer technically focused.\n"
                .to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_is_none() {
        assert!(prompt_addon(CulturalWisdomLevel::Off).is_none());
    }

    #[test]
    fn light_nonempty() {
        assert!(
            prompt_addon(CulturalWisdomLevel::Light)
                .unwrap()
                .contains("cultural wisdom")
        );
    }
}
