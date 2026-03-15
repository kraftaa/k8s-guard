use crate::model::{Confidence, LabelExpression, Severity, WorkloadSpec};
use crate::rules::base_finding;
use crate::rules::traits::Rule;

pub struct SelectorChangedRule;

impl Rule for SelectorChangedRule {
    fn id(&self) -> &'static str {
        "KDX011"
    }

    fn check(&self, old: &WorkloadSpec, new: &WorkloadSpec) -> Vec<crate::model::Finding> {
        let selector_changed = old.selector_labels != new.selector_labels
            || old.selector_expressions != new.selector_expressions;
        let template_mismatch =
            !new.selector_labels.is_empty() && new.selector_labels != new.template_labels;
        let expressions_unsatisfied = !new.selector_expressions.is_empty()
            && !expressions_match_template(&new.selector_expressions, &new.template_labels);

        if !selector_changed && !template_mismatch && !expressions_unsatisfied {
            return Vec::new();
        }

        let old_labels = fmt_labels(&old.selector_labels);
        let new_labels = fmt_labels(&new.selector_labels);
        let old_exprs = fmt_exprs(&old.selector_expressions);
        let new_exprs = fmt_exprs(&new.selector_expressions);

        vec![base_finding(
            self.id(),
            Severity::High,
            Confidence::Medium,
            old,
            new,
            None,
            "spec.selector.matchLabels".to_string(),
            "Pod selector changed",
            merge_opt(old_labels, old_exprs),
            merge_opt(new_labels, new_exprs),
            vec![
                "Deployment may not manage existing pods",
                "Services may target wrong or no pods",
                "Rollout could orphan pods or route zero traffic",
            ],
            "Selectors are typically immutable; changing them or letting them diverge from template labels can orphan or misroute traffic.",
            vec![
                "avoid changing selectors; create a new workload instead",
                "if intentional, align template labels and Service selectors",
                "ensure matchExpressions still match template labels",
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

fn fmt_exprs(exprs: &[LabelExpression]) -> Option<String> {
    if exprs.is_empty() {
        None
    } else {
        Some(
            exprs
                .iter()
                .map(|e| {
                    if e.values.is_empty() {
                        format!("{} {}", e.key, e.operator)
                    } else {
                        format!("{} {} [{}]", e.key, e.operator, e.values.join(","))
                    }
                })
                .collect::<Vec<_>>()
                .join("; "),
        )
    }
}

fn merge_opt(a: Option<String>, b: Option<String>) -> Option<String> {
    match (a, b) {
        (None, None) => None,
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (Some(x), Some(y)) => Some(format!("{x}; {y}")),
    }
}

fn expressions_match_template(
    exprs: &[LabelExpression],
    labels: &std::collections::BTreeMap<String, String>,
) -> bool {
    exprs.iter().all(|e| {
        match e.operator.as_str() {
            "In" => labels
                .get(&e.key)
                .map(|v| e.values.contains(v))
                .unwrap_or(false),
            "NotIn" => labels
                .get(&e.key)
                .map(|v| !e.values.contains(v))
                .unwrap_or(true),
            "Exists" => labels.contains_key(&e.key),
            "DoesNotExist" => !labels.contains_key(&e.key),
            _ => true, // unknown operator: do not block rollout but mark changed
        }
    })
}
