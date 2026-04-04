//! Static approximate USD pricing (MVP). Token counts are inferred from UTF-8 length (`chars / 4`).

use super::providers::ProviderKind;

fn approx_tokens(chars: usize) -> f64 {
    (chars as f64 / 4.0).max(1.0)
}

/// `(input_usd_per_million_tokens, output_usd_per_million_tokens)`.
fn price_per_million(kind: ProviderKind, model: &str) -> (f64, f64) {
    let m = model.to_ascii_lowercase();
    match kind {
        ProviderKind::Ollama => (0.0, 0.0),
        ProviderKind::Anthropic => {
            if m.contains("haiku") {
                (0.80, 4.0)
            } else if m.contains("opus") {
                (15.0, 75.0)
            } else {
                (3.0, 15.0)
            }
        }
        ProviderKind::OpenAi => {
            if m.contains("mini") || m.contains("4o-mini") {
                (0.15, 0.60)
            } else if m.contains("gpt-4") {
                (5.0, 15.0)
            } else {
                (0.50, 1.50)
            }
        }
        ProviderKind::Gemini => {
            if m.contains("flash") {
                (0.075, 0.30)
            } else {
                (1.25, 5.0)
            }
        }
        ProviderKind::OpenRouter | ProviderKind::Groq => (0.20, 0.60),
        ProviderKind::AzureOpenAi => (5.0, 15.0),
    }
}

/// Rough USD cost from character counts (not real token usage from APIs).
pub fn approx_cost_usd(
    kind: ProviderKind,
    model: &str,
    input_chars: usize,
    output_chars: usize,
) -> f64 {
    let (pi, po) = price_per_million(kind, model);
    let tin = approx_tokens(input_chars);
    let tout = approx_tokens(output_chars);
    (tin * pi + tout * po) / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ollama_free() {
        assert_eq!(
            approx_cost_usd(ProviderKind::Ollama, "llama3", 1000, 500),
            0.0
        );
    }

    #[test]
    fn openai_mini_positive() {
        let c = approx_cost_usd(ProviderKind::OpenAi, "gpt-4o-mini", 4000, 4000);
        assert!(c > 0.0 && c < 0.01);
    }
}
