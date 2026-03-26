use crate::model::{Confidence, Severity, WorkloadSpec};
use crate::rules::traits::Rule;
use crate::rules::{
    base_finding, container_field_path, container_label, mem_to_string, pair_containers,
};

pub struct MemoryLimitReducedRule;

impl Rule for MemoryLimitReducedRule {
    fn id(&self) -> &'static str {
        "KDX001"
    }

    fn check(&self, old: &WorkloadSpec, new: &WorkloadSpec) -> Vec<crate::model::Finding> {
        let mut findings = Vec::new();
        for (oc, nc) in pair_containers(old, new) {
            if let (Some(old_mem), Some(new_mem)) = (oc.memory_limit_bytes, nc.memory_limit_bytes)
                && new_mem < old_mem
            {
                let ratio = new_mem as f64 / old_mem as f64;
                let severity = if ratio <= 0.5 || (old_mem >= gib(1) && new_mem <= mib(512)) {
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
                        container_field_path(nc, "resources.limits.memory"),
                        "Memory limit reduced",
                    Some(mem_to_string(old_mem)),
                    Some(mem_to_string(new_mem)),
                    vec!["OOMKilled", "CrashLoopBackOff"],
                    "Reducing memory limits increases the chance the container is killed when usage spikes.",
                    vec![
                        "restore previous memory limit",
                        "review request/limit pair together",
                        "confirm change was intentional",
                    ],
                ));
            }
        }
        findings
    }
}

fn gib(v: i64) -> i64 {
    v * 1024 * 1024 * 1024
}

fn mib(v: i64) -> i64 {
    v * 1024 * 1024
}
