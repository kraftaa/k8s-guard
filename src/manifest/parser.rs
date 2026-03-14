use crate::manifest::normalize_workload;
use crate::model::WorkloadSpec;
use anyhow::Context;
use serde::Deserialize;
use serde_yaml::Value;
use std::path::Path;

pub fn load_workloads(path: &Path) -> anyhow::Result<Vec<WorkloadSpec>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read manifest file {}", path.display()))?;

    let mut specs = Vec::new();
    for doc in serde_yaml::Deserializer::from_str(&content) {
        let value: Value =
            serde_yaml::Value::deserialize(doc).context("failed to parse manifest as YAML")?;
        if let Some(workload) = normalize_workload(&value)? {
            specs.push(workload);
        }
    }
    Ok(specs)
}
