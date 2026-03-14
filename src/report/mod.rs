mod json;
mod text;

pub use json::render_json;
pub use text::render_text;

use crate::model::{Finding, OverallRisk, WorkloadKey};

#[derive(Debug, serde::Serialize)]
pub struct ResourceResult {
    pub key: WorkloadKey,
    pub findings: Vec<Finding>,
    pub overall_risk: OverallRisk,
    pub notes: Vec<String>,
}
