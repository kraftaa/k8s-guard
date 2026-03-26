use crate::model::{Confidence, Severity, WorkloadSpec};
use crate::rules::traits::Rule;
use crate::rules::{base_finding, container_field_path, container_label, pair_containers};

pub struct EnvRemovedRule;

impl Rule for EnvRemovedRule {
    fn id(&self) -> &'static str {
        "KDX007"
    }

    fn check(&self, old: &WorkloadSpec, new: &WorkloadSpec) -> Vec<crate::model::Finding> {
        let mut findings = Vec::new();
        for (oc, nc) in pair_containers(old, new) {
            for key in oc.env.keys() {
                if !nc.env.contains_key(key) {
                    let severity = if looks_critical(key) {
                        Severity::High
                    } else {
                        Severity::Medium
                    };
                    findings.push(base_finding(
                        self.id(),
                        severity,
                        Confidence::Medium,
                        old,
                        new,
                        Some(container_label(nc)),
                        container_field_path(nc, &format!("env[{}]", key)),
                        "Required env var removed",
                        Some(format!("present: {}", key)),
                        Some("removed".to_string()),
                        vec!["Application startup failure", "CrashLoopBackOff"],
                        "Removing required environment variables is a common cause of configuration failures.",
                        vec![
                            "restore the removed env var or confirm code change supports removal",
                            "coordinate env renames with application changes",
                        ],
                    ));
                }
            }
        }
        findings
    }
}

fn looks_critical(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    upper.starts_with("DB_")
        || upper.contains("DATABASE")
        || upper.starts_with("REDIS")
        || upper.contains("API_KEY")
        || upper.contains("SECRET")
        || upper.contains("BROKER")
}
