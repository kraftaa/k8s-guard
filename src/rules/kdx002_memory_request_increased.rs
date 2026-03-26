use crate::model::{Confidence, Severity, WorkloadSpec};
use crate::rules::traits::Rule;
use crate::rules::{
    base_finding, container_field_path, container_label, mem_to_string, pair_containers,
};

pub struct MemoryRequestIncreasedRule;

impl Rule for MemoryRequestIncreasedRule {
    fn id(&self) -> &'static str {
        "KDX002"
    }

    fn check(&self, old: &WorkloadSpec, new: &WorkloadSpec) -> Vec<crate::model::Finding> {
        let mut findings = Vec::new();
        let replicas_increased = match (old.replicas, new.replicas) {
            (Some(o), Some(n)) => n > o,
            _ => false,
        };

        for (oc, nc) in pair_containers(old, new) {
            if let (Some(old_mem), Some(new_mem)) =
                (oc.memory_request_bytes, nc.memory_request_bytes)
                && new_mem > old_mem
            {
                let ratio = new_mem as f64 / old_mem as f64;
                if ratio >= 1.5 {
                    let severity = if ratio >= 2.0 && replicas_increased {
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
                            container_field_path(nc, "resources.requests.memory"),
                            "Memory request increased sharply",
                        Some(mem_to_string(old_mem)),
                        Some(mem_to_string(new_mem)),
                        vec!["Pending pods", "Rollout may stall"],
                        "Scheduler places pods using requests; sharp increases can make pods unschedulable.",
                        vec![
                            "increase requests gradually",
                            "verify cluster capacity for new request",
                            "reduce request if overly conservative",
                        ],
                    ));
                }
            }
        }
        findings
    }
}
