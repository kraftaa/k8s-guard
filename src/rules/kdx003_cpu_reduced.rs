use crate::model::{Confidence, Severity, WorkloadSpec};
use crate::rules::traits::Rule;
use crate::rules::{
    base_finding, container_field_path, container_label, cpu_to_string, pair_containers,
};

pub struct CpuReducedRule;

impl Rule for CpuReducedRule {
    fn id(&self) -> &'static str {
        "KDX003"
    }

    fn check(&self, old: &WorkloadSpec, new: &WorkloadSpec) -> Vec<crate::model::Finding> {
        let mut findings = Vec::new();
        for (oc, nc) in pair_containers(old, new) {
            // Requests
            if let (Some(or), Some(nr)) = (oc.cpu_request_millis, nc.cpu_request_millis) {
                let ratio = nr as f64 / or as f64;
                if ratio <= 0.5 {
                    findings.push(base_finding(
                        self.id(),
                        Severity::Medium,
                        Confidence::Medium,
                        old,
                        new,
                        Some(container_label(nc)),
                        container_field_path(nc, "resources.requests.cpu"),
                        "CPU request reduced",
                        Some(cpu_to_string(or)),
                        Some(cpu_to_string(nr)),
                        vec!["Slower startup", "Probe failures possible"],
                        "Lower CPU requests can delay startup and cause probe timeouts under load.",
                        vec![
                            "restore prior CPU request",
                            "loosen probe timing if startup slows",
                        ],
                    ));
                }
            }
            // Limits
            if let (Some(ol), Some(nl)) = (oc.cpu_limit_millis, nc.cpu_limit_millis) {
                let ratio = nl as f64 / ol as f64;
                if ratio <= 0.5 {
                    findings.push(base_finding(
                        self.id(),
                        Severity::Medium,
                        Confidence::Medium,
                        old,
                        new,
                        Some(container_label(nc)),
                        container_field_path(nc, "resources.limits.cpu"),
                        "CPU limit reduced",
                        Some(cpu_to_string(ol)),
                        Some(cpu_to_string(nl)),
                        vec!["Throttling under load", "Probe failures possible"],
                        "Tight CPU limits increase throttling risk during startup and warm-up.",
                        vec![
                            "restore prior CPU limit",
                            "validate performance with lower limit",
                        ],
                    ));
                }
            }
        }
        findings
    }
}
