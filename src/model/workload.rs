use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, serde::Serialize)]
pub struct WorkloadKey {
    pub kind: String,
    pub name: String,
    pub namespace: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct WorkloadSpec {
    pub key: WorkloadKey,
    pub replicas: Option<i32>,
    pub containers: Vec<ContainerSpecLite>,
    pub node_selector: BTreeMap<String, String>,
    pub tolerations: Vec<TolerationLite>,
    pub has_required_node_affinity: bool,
    pub image_pull_secrets: Vec<String>,
    pub selector_labels: BTreeMap<String, String>,
    pub template_labels: BTreeMap<String, String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ContainerSpecLite {
    pub name: String,
    pub image: Option<String>,

    pub cpu_request_millis: Option<i64>,
    pub cpu_limit_millis: Option<i64>,
    pub memory_request_bytes: Option<i64>,
    pub memory_limit_bytes: Option<i64>,

    pub readiness_probe: Option<ProbeLite>,
    pub liveness_probe: Option<ProbeLite>,
    pub startup_probe: Option<ProbeLite>,

    pub env: BTreeMap<String, EnvValueLite>,
    pub ports: Vec<ContainerPortLite>,

    pub secret_refs: BTreeSet<String>,
    pub config_map_refs: BTreeSet<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct EnvValueLite {
    pub value: Option<String>,
    pub from: Option<EnvFrom>,
}

#[allow(dead_code, clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum EnvFrom {
    SecretKeyRef { name: String, key: Option<String> },
    ConfigMapKeyRef { name: String, key: Option<String> },
    FieldRef,
    ResourceFieldRef,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ContainerPortLite {
    pub container_port: Option<i32>,
    pub name: Option<String>,
    pub protocol: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ProbeLite {
    pub probe_type: String, // http,tcp,exec
    pub path: Option<String>,
    pub port: Option<String>,
    pub timeout_seconds: Option<i32>,
    pub period_seconds: Option<i32>,
    pub failure_threshold: Option<i32>,
    pub success_threshold: Option<i32>,
    pub initial_delay_seconds: Option<i32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TolerationLite {
    pub key: Option<String>,
    pub operator: Option<String>,
    pub value: Option<String>,
    pub effect: Option<String>,
}
