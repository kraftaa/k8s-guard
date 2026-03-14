use crate::model::{Confidence, Severity, WorkloadSpec};
use crate::rules::base_finding;
use crate::rules::traits::Rule;

pub struct SchedulingTightenedRule;

impl Rule for SchedulingTightenedRule {
    fn id(&self) -> &'static str {
        "KDX009"
    }

    fn check(&self, old: &WorkloadSpec, new: &WorkloadSpec) -> Vec<crate::model::Finding> {
        let mut findings = Vec::new();

        let selectors_tightened =
            new.node_selector.len() > old.node_selector.len() && !new.node_selector.is_empty();
        let affinity_added = !old.has_required_node_affinity && new.has_required_node_affinity;

        if selectors_tightened || affinity_added {
            let mut changes = Vec::new();
            if selectors_tightened {
                changes.push(format!(
                    "nodeSelector keys {} -> {}",
                    old.node_selector.len(),
                    new.node_selector.len()
                ));
            }
            if affinity_added {
                changes.push("required node affinity added".to_string());
            }

            findings.push(base_finding(
                self.id(),
                Severity::High,
                Confidence::High,
                old,
                new,
                None,
                "spec.template.spec.scheduling".to_string(),
                "Scheduling constraints tightened",
                None,
                Some(changes.join("; ")),
                vec!["Pods may stay Pending", "Rollout may stall"],
                "Tighter scheduling constraints reduce the set of eligible nodes and often lead to Pending pods.",
                vec![
                    "loosen nodeSelector or affinity",
                    "confirm matching nodes exist before rollout",
                ],
            ));
        }
        findings
    }
}
