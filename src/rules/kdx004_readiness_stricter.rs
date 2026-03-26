use crate::model::{Confidence, Finding, Severity, WorkloadSpec};
use crate::rules::traits::Rule;
use crate::rules::{base_finding, container_field_path, container_label, pair_containers};

pub struct ReadinessProbeStricterRule;

impl Rule for ReadinessProbeStricterRule {
    fn id(&self) -> &'static str {
        "KDX004"
    }

    fn check(&self, old: &WorkloadSpec, new: &WorkloadSpec) -> Vec<Finding> {
        let mut findings = Vec::new();
        for (oc, nc) in pair_containers(old, new) {
            match (&oc.readiness_probe, &nc.readiness_probe) {
                (Some(op), Some(np)) => {
                    let (score, path_or_port_changed) = probe_strictness(op, np);
                    if score >= 3 || path_or_port_changed {
                        let severity = if path_or_port_changed {
                            Severity::High
                        } else {
                            Severity::Medium
                        };
                        findings.push(base_finding(
                            self.id(),
                            severity,
                            Confidence::High,
                            old,
                            new,
                            Some(container_label(nc)),
                            container_field_path(nc, "readinessProbe"),
                            "Readiness probe became stricter",
                            Some(summarize_probe(op)),
                            Some(summarize_probe(np)),
                            vec![
                                "Pod may never become Ready",
                                "Rollout can stall",
                                "Traffic may not reach new pods",
                            ],
                            "Tighter readiness checks reduce margin for slow startup or transient response spikes.",
                            vec![
                                "restore previous readiness settings",
                                "avoid tightening multiple probe fields at once",
                                "verify endpoint path and port",
                            ],
                        ));
                    }
                }
                (None, Some(np)) => {
                    findings.push(base_finding(
                        self.id(),
                        Severity::Medium,
                        Confidence::High,
                        old,
                        new,
                        Some(container_label(nc)),
                        container_field_path(nc, "readinessProbe"),
                        "Readiness probe added",
                        None,
                        Some(summarize_probe(np)),
                        vec!["New readiness gate introduced", "Rollout could stall"],
                        "Adding a readiness probe where none existed can prevent traffic if the endpoint is not ready.",
                        vec![
                            "validate readiness endpoint before rollout",
                            "relax timings if startup is slow",
                        ],
                    ));
                }
                _ => {}
            }
        }
        findings
    }
}

fn probe_strictness(old: &crate::model::ProbeLite, new: &crate::model::ProbeLite) -> (i32, bool) {
    let mut score = 0;
    let mut path_or_port = false;

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
    if old.path != new.path {
        score += 3;
        path_or_port = true;
    }
    if old.port != new.port {
        score += 3;
        path_or_port = true;
    }

    (score, path_or_port)
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
