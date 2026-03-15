use crate::model::{Confidence, Finding, Severity, WorkloadSpec};
use crate::rules::traits::Rule;
use crate::rules::{base_finding, format_field_path, pair_containers};

pub struct LivenessWithoutStartupRule;

impl Rule for LivenessWithoutStartupRule {
    fn id(&self) -> &'static str {
        "KDX005"
    }

    fn check(&self, old: &WorkloadSpec, new: &WorkloadSpec) -> Vec<Finding> {
        let mut findings = Vec::new();
        for (oc, nc) in pair_containers(old, new) {
            if nc.startup_probe.is_some() {
                continue;
            }
            if let Some(nl) = &nc.liveness_probe {
                let stricter = match &oc.liveness_probe {
                    Some(ol) => {
                        probe_strictness(ol, nl) >= 2 || ol.path != nl.path || ol.port != nl.port
                    }
                    None => true,
                };
                if stricter {
                    findings.push(base_finding(
                        self.id(),
                        Severity::High,
                        Confidence::High,
                        old,
                        new,
                        Some(nc.name.clone()),
                        format_field_path(&nc.name, "livenessProbe"),
                        "Liveness probe stricter without startupProbe",
                        oc.liveness_probe.as_ref().map(summarize_probe),
                        Some(summarize_probe(nl)),
                        vec!["Restart loop / CrashLoopBackOff", "Container may die before startup completes"],
                        "Stricter liveness probes without a startup probe can kill slow-starting containers before they are ready.",
                        vec![
                            "add a startupProbe",
                            "increase liveness initialDelaySeconds or timeoutSeconds",
                            "restore previous liveness settings",
                        ],
                    ));
                }
            }
        }
        findings
    }
}

fn probe_strictness(old: &crate::model::ProbeLite, new: &crate::model::ProbeLite) -> i32 {
    let mut score = 0;
    if let (Some(o), Some(n)) = (old.timeout_seconds, new.timeout_seconds)
        && n < o
    {
        score += 2;
    }
    if let (Some(o), Some(n)) = (old.failure_threshold, new.failure_threshold)
        && n < o
    {
        score += 2;
    }
    if let (Some(o), Some(n)) = (old.period_seconds, new.period_seconds)
        && n < o
    {
        score += 1;
    }
    score
}

fn summarize_probe(p: &crate::model::ProbeLite) -> String {
    let timeout = p
        .timeout_seconds
        .map(|v| format!("timeout={}s", v))
        .unwrap_or_else(|| "timeout=default".to_string());
    let period = p
        .period_seconds
        .map(|v| format!("period={}s", v))
        .unwrap_or_else(|| "period=default".to_string());
    let path = p.path.clone().unwrap_or_else(|| "path?".to_string());
    let port = p.port.clone().unwrap_or_else(|| "port?".to_string());
    format!("{} {} {} {path}:{port}", p.probe_type, timeout, period)
}
