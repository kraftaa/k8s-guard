mod kdx001_memory_limit_reduced;
mod kdx002_memory_request_increased;
mod kdx003_cpu_reduced;
mod kdx004_readiness_stricter;
mod kdx005_liveness_without_startup;
mod kdx006_image_pull_risk;
mod kdx007_env_removed;
mod kdx008_config_ref_changed;
mod kdx009_scheduling_tightened;
mod kdx010_replica_pressure;
mod kdx011_selector_changed;
pub mod traits;
mod util;

use crate::model::Finding;
use crate::model::WorkloadSpec;
use traits::Rule;
pub use util::*;

pub fn run_rules(old: &WorkloadSpec, new: &WorkloadSpec, experimental: bool) -> Vec<Finding> {
    let mut rules: Vec<Box<dyn Rule>> = vec![
        Box::new(kdx001_memory_limit_reduced::MemoryLimitReducedRule),
        Box::new(kdx002_memory_request_increased::MemoryRequestIncreasedRule),
        Box::new(kdx003_cpu_reduced::CpuReducedRule),
        Box::new(kdx004_readiness_stricter::ReadinessProbeStricterRule),
        Box::new(kdx005_liveness_without_startup::LivenessWithoutStartupRule),
        Box::new(kdx006_image_pull_risk::ImagePullRiskRule),
        Box::new(kdx007_env_removed::EnvRemovedRule),
        Box::new(kdx008_config_ref_changed::ConfigRefChangedRule),
        Box::new(kdx009_scheduling_tightened::SchedulingTightenedRule),
        Box::new(kdx010_replica_pressure::ReplicaPressureRule),
    ];
    if experimental {
        rules.push(Box::new(kdx011_selector_changed::SelectorChangedRule));
    }

    let mut findings = Vec::new();
    for rule in rules {
        findings.extend(rule.check(old, new));
    }
    findings
}
