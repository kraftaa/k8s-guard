mod json;
mod text;

pub use json::{render_json, render_json_string};
pub use text::{render_text, render_text_string};

use crate::model::{Finding, OverallRisk, WorkloadKey};

#[derive(Debug, serde::Serialize)]
pub struct ResourceResult {
    pub key: WorkloadKey,
    pub findings: Vec<Finding>,
    pub overall_risk: OverallRisk,
    pub notes: Vec<String>,
    pub removed: bool,
}
