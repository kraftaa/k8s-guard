use crate::model::{Confidence, ContainerSpecLite, Finding, OverallRisk, Severity, WorkloadSpec};

pub fn pair_containers<'a>(
    old: &'a WorkloadSpec,
    new: &'a WorkloadSpec,
) -> Vec<(&'a ContainerSpecLite, &'a ContainerSpecLite)> {
    let mut pairs = Vec::new();
    for n in new.containers.iter().chain(new.init_containers.iter()) {
        if let Some(o) = old
            .containers
            .iter()
            .chain(old.init_containers.iter())
            .find(|c| c.name == n.name && c.is_init == n.is_init)
        {
            pairs.push((o, n));
        }
    }
    pairs
}

pub fn mem_to_string(bytes: i64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    if bytes as f64 >= GIB {
        format!("{:.0}Gi", bytes as f64 / GIB)
    } else if bytes as f64 >= MIB {
        format!("{:.0}Mi", bytes as f64 / MIB)
    } else {
        format!("{}B", bytes)
    }
}

pub fn cpu_to_string(millis: i64) -> String {
    if millis % 1000 == 0 {
        format!("{:.0}", millis as f64 / 1000.0)
    } else {
        format!("{}m", millis)
    }
}

pub fn format_field_path(container: &str, suffix: &str) -> String {
    format!("spec.template.spec.containers[{}].{}", container, suffix)
}

pub fn container_label(c: &ContainerSpecLite) -> String {
    if c.is_init {
        format!("init:{}", c.name)
    } else {
        c.name.clone()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn base_finding(
    rule_id: &str,
    severity: Severity,
    confidence: Confidence,
    _old: &WorkloadSpec,
    new: &WorkloadSpec,
    container: Option<String>,
    field_path: String,
    title: &str,
    old_value: Option<String>,
    new_value: Option<String>,
    likely_impact: Vec<&str>,
    why_it_matters: &str,
    suggested_fix: Vec<&str>,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity,
        confidence,
        resource_kind: new.key.kind.clone(),
        resource_name: new.key.name.clone(),
        namespace: new.key.namespace.clone(),
        container,
        field_path,
        title: title.to_string(),
        old_value,
        new_value,
        likely_impact: likely_impact.into_iter().map(|s| s.to_string()).collect(),
        why_it_matters: why_it_matters.to_string(),
        suggested_fix: suggested_fix.into_iter().map(|s| s.to_string()).collect(),
    }
}

pub fn score_findings(findings: &[Finding]) -> OverallRisk {
    let mut score = 0;
    let mut high_count = 0;
    for f in findings {
        match f.severity {
            Severity::Low => score += 1,
            Severity::Medium => score += 3,
            Severity::High => {
                score += 6;
                high_count += 1;
            }
        }
    }
    if high_count >= 2 {
        return OverallRisk::High;
    }
    match score {
        0 => OverallRisk::Safe,
        1..=5 => OverallRisk::Low,
        6..=11 => OverallRisk::Medium,
        _ => OverallRisk::High,
    }
}
