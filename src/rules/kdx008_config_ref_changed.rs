use crate::model::{Confidence, Severity, WorkloadSpec};
use crate::rules::traits::Rule;
use crate::rules::{base_finding, format_field_path, pair_containers};

pub struct ConfigRefChangedRule;

impl Rule for ConfigRefChangedRule {
    fn id(&self) -> &'static str {
        "KDX008"
    }

    fn check(&self, old: &WorkloadSpec, new: &WorkloadSpec) -> Vec<crate::model::Finding> {
        let mut findings = Vec::new();
        for (oc, nc) in pair_containers(old, new) {
            let removed_secret: Vec<_> = oc
                .secret_refs
                .difference(&nc.secret_refs)
                .cloned()
                .collect();
            let added_secret: Vec<_> = nc
                .secret_refs
                .difference(&oc.secret_refs)
                .cloned()
                .collect();

            let removed_cm: Vec<_> = oc
                .config_map_refs
                .difference(&nc.config_map_refs)
                .cloned()
                .collect();
            let added_cm: Vec<_> = nc
                .config_map_refs
                .difference(&oc.config_map_refs)
                .cloned()
                .collect();

            if !removed_secret.is_empty()
                || !removed_cm.is_empty()
                || !added_secret.is_empty()
                || !added_cm.is_empty()
            {
                let severity = if !removed_secret.is_empty() || !removed_cm.is_empty() {
                    Severity::High
                } else {
                    Severity::Medium
                };

                let mut old_val = Vec::new();
                if !removed_secret.is_empty() {
                    old_val.push(format!("secrets removed: {}", removed_secret.join(",")));
                }
                if !removed_cm.is_empty() {
                    old_val.push(format!("configMaps removed: {}", removed_cm.join(",")));
                }
                let mut new_val = Vec::new();
                if !added_secret.is_empty() {
                    new_val.push(format!("secrets added: {}", added_secret.join(",")));
                }
                if !added_cm.is_empty() {
                    new_val.push(format!("configMaps added: {}", added_cm.join(",")));
                }

                findings.push(base_finding(
                    self.id(),
                    severity,
                    Confidence::High,
                    old,
                    new,
                    Some(nc.name.clone()),
                    format_field_path(&nc.name, "config/secret references"),
                    "Config/Secret reference changed",
                    if old_val.is_empty() { None } else { Some(old_val.join("; ")) },
                    if new_val.is_empty() { None } else { Some(new_val.join("; ")) },
                    vec![
                        "CreateContainerConfigError",
                        "Application startup failure",
                    ],
                    "Changing Secret or ConfigMap references without deploying those configs first commonly breaks startups.",
                    vec![
                        "verify referenced objects and keys exist in target namespace",
                        "deploy config changes before workload rollout",
                    ],
                ));
            }
        }
        findings
    }
}
