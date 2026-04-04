use serde::{Deserialize, Serialize};

/// Serializable plan from LLM JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub steps: Vec<PlanStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanStep {
    /// Stable id (e.g. "1", "step-a").
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub suggested_action: Option<String>,
}

impl Plan {
    pub fn single_manual(description: impl Into<String>) -> Self {
        Plan {
            steps: vec![PlanStep {
                id: "1".into(),
                description: description.into(),
                suggested_action: None,
            }],
        }
    }
}
