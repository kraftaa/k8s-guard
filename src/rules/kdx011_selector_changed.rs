use crate::model::{Confidence, Severity, WorkloadSpec};
use crate::rules::base_finding;
use crate::rules::traits::Rule;

pub struct SelectorChangedRule;

impl Rule for SelectorChangedRule {
    fn id(&self) -> &'static str {
        "KDX011"
    }

    fn check(&self, old: &WorkloadSpec, new: &WorkloadSpec) -> Vec<crate::model::Finding> {
        let selector_changed = old.selector_labels != new.selector_labels;
        let template_mismatch = !new.selector_labels.is_empty()
            && new.selector_labels != new.template_labels;

        if !selector_changed && !template_mismatch {
            return Vec::new();
        }

        let old_labels = fmt_labels(&old.selector_labels);
        let new_labels = fmt_labels(&new.selector_labels);

        vec![base_finding(
            self.id(),
            Severity::High,
            Confidence::Medium,
            old,
            new,
            None,
            "spec.selector.matchLabels".to_string(),
            "Pod selector changed",
            old_labels,
            new_labels,
            vec![
                "Deployment may not manage existing pods",
                "Services may target wrong or no pods",
                "Rollout could orphan pods or route zero traffic",
            ],
            "Selectors are typically immutable; changing them or letting them diverge from template labels can orphan or misroute traffic.",
            vec![
                "avoid changing selectors; create a new workload instead",
                "if intentional, align template labels and Service selectors",
            ],
        )]
    }
}

fn fmt_labels(m: &std::collections::BTreeMap<String, String>) -> Option<String> {
    if m.is_empty() {
        None
    } else {
        Some(
            m.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}
