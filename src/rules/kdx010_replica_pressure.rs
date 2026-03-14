use crate::model::{Confidence, Severity, WorkloadSpec};
use crate::rules::traits::Rule;
use crate::rules::{base_finding, pair_containers};

pub struct ReplicaPressureRule;

impl Rule for ReplicaPressureRule {
    fn id(&self) -> &'static str {
        "KDX010"
    }

    fn check(&self, old: &WorkloadSpec, new: &WorkloadSpec) -> Vec<crate::model::Finding> {
        let mut findings = Vec::new();
        let (old_rep, new_rep) = match (old.replicas, new.replicas) {
            (Some(o), Some(n)) => (o, n),
            _ => return findings,
        };

        let replica_jump = new_rep >= old_rep * 2 || (new_rep - old_rep) >= 3;
        if !replica_jump {
            return findings;
        }

        let mut pressure_signals = 0;
        if memory_requests_increased(old, new) {
            pressure_signals += 1;
        }
        if cpu_requests_increased(old, new) {
            pressure_signals += 1;
        }
        if scheduling_tightened(old, new) {
            pressure_signals += 1;
        }
        if readiness_stricter(old, new) {
            pressure_signals += 1;
        }

        if pressure_signals > 0 {
            let severity = if pressure_signals >= 2 {
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
                None,
                "spec.replicas".to_string(),
                "Replica increase with pressure signals",
                Some(old_rep.to_string()),
                Some(new_rep.to_string()),
                vec!["Rollout may stall", "Insufficient capacity", "Partial availability"],
                "A large replica increase combined with resource or scheduling pressure often leaves pods Pending or NotReady.",
                vec![
                    "scale in stages instead of a single large jump",
                    "validate capacity before scaling",
                    "avoid combining replica bumps with tighter probes or scheduling",
                ],
            ));
        }

        findings
    }
}

fn memory_requests_increased(old: &WorkloadSpec, new: &WorkloadSpec) -> bool {
    pair_containers(old, new).iter().any(|(oc, nc)| {
        if let (Some(o), Some(n)) = (oc.memory_request_bytes, nc.memory_request_bytes) {
            n > o && (n as f64 / o as f64) >= 1.25
        } else {
            false
        }
    })
}

fn cpu_requests_increased(old: &WorkloadSpec, new: &WorkloadSpec) -> bool {
    pair_containers(old, new).iter().any(|(oc, nc)| {
        if let (Some(o), Some(n)) = (oc.cpu_request_millis, nc.cpu_request_millis) {
            n > o && (n as f64 / o as f64) >= 1.25
        } else {
            false
        }
    })
}

fn readiness_stricter(old: &WorkloadSpec, new: &WorkloadSpec) -> bool {
    pair_containers(old, new).iter().any(|(oc, nc)| {
        match (&oc.readiness_probe, &nc.readiness_probe) {
        (Some(op), Some(np)) => {
            if let (Some(o), Some(nv)) = (op.timeout_seconds, np.timeout_seconds) && nv < o {
                return true;
            }
            if let (Some(o), Some(nv)) = (op.failure_threshold, np.failure_threshold) && nv < o {
                return true;
            }
            if let (Some(o), Some(nv)) = (op.period_seconds, np.period_seconds) && nv < o {
                return true;
            }
            op.path != np.path || op.port != np.port
        }
            (None, Some(_)) => true,
            _ => false,
        }
    })
}

fn scheduling_tightened(old: &WorkloadSpec, new: &WorkloadSpec) -> bool {
    (new.node_selector.len() > old.node_selector.len() && !new.node_selector.is_empty())
        || (!old.has_required_node_affinity && new.has_required_node_affinity)
}
