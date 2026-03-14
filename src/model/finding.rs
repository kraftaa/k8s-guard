use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Low,
    Medium,
    High,
}

impl Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "LOW"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::High => write!(f, "HIGH"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Confidence::Low => write!(f, "LOW"),
            Confidence::Medium => write!(f, "MEDIUM"),
            Confidence::High => write!(f, "HIGH"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OverallRisk {
    Safe,
    Low,
    Medium,
    High,
}

impl Display for OverallRisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OverallRisk::Safe => write!(f, "SAFE"),
            OverallRisk::Low => write!(f, "LOW"),
            OverallRisk::Medium => write!(f, "MEDIUM"),
            OverallRisk::High => write!(f, "HIGH"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Finding {
    pub rule_id: String,
    pub severity: Severity,
    pub confidence: Confidence,

    pub resource_kind: String,
    pub resource_name: String,
    pub namespace: Option<String>,

    pub container: Option<String>,
    pub field_path: String,

    pub title: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,

    pub likely_impact: Vec<String>,
    pub why_it_matters: String,
    pub suggested_fix: Vec<String>,
}
