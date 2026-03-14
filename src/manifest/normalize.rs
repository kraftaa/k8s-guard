use crate::model::{
    ContainerPortLite, ContainerSpecLite, EnvFrom, EnvValueLite, ProbeLite, TolerationLite,
    WorkloadKey, WorkloadSpec,
};
use anyhow::anyhow;
use serde_yaml::{Mapping, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub fn normalize_workload(value: &Value) -> anyhow::Result<Option<WorkloadSpec>> {
    let root = match value.as_mapping() {
        Some(m) => m,
        None => return Ok(None),
    };

    let kind = get_string(root, "kind").unwrap_or_default();
    if !matches!(kind.as_str(), "Deployment" | "StatefulSet" | "DaemonSet") {
        return Ok(None);
    }

    let metadata = root
        .get(&Value::from("metadata"))
        .and_then(Value::as_mapping)
        .ok_or_else(|| anyhow!("missing metadata block"))?;

    let name =
        get_string(metadata, "name").ok_or_else(|| anyhow!("resource missing metadata.name"))?;
    let namespace = get_string(metadata, "namespace");

    let spec = root
        .get(&Value::from("spec"))
        .and_then(Value::as_mapping)
        .ok_or_else(|| anyhow!("missing spec block for {}", name))?;

    let replicas = get_i64(spec, "replicas").map(|v| v as i32);
    let template_spec = spec
        .get(&Value::from("template"))
        .and_then(Value::as_mapping)
        .and_then(|t| t.get(&Value::from("spec")).and_then(Value::as_mapping))
        .ok_or_else(|| anyhow!("missing spec.template.spec for {}", name))?;

    let volumes = collect_volumes(template_spec);
    let image_pull_secrets = collect_image_pull_secrets(template_spec);
    let node_selector = collect_node_selector(template_spec);
    let tolerations = collect_tolerations(template_spec);
    let has_required_node_affinity = detect_required_node_affinity(template_spec);

    let containers_val = template_spec
        .get(&Value::from("containers"))
        .and_then(Value::as_sequence)
        .ok_or_else(|| anyhow!("spec.template.spec.containers missing for {}", name))?;

    let mut containers = Vec::new();
    for c in containers_val {
        containers.push(parse_container(c, &volumes)?);
    }

    let workload = WorkloadSpec {
        key: WorkloadKey {
            kind,
            name,
            namespace,
        },
        replicas,
        containers,
        node_selector,
        tolerations,
        has_required_node_affinity,
        image_pull_secrets,
    };

    Ok(Some(workload))
}

fn collect_image_pull_secrets(spec: &Mapping) -> Vec<String> {
    spec.get(&Value::from("imagePullSecrets"))
        .and_then(Value::as_sequence)
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_mapping())
                .filter_map(|m| get_string(m, "name"))
                .collect()
        })
        .unwrap_or_default()
}

fn collect_node_selector(spec: &Mapping) -> BTreeMap<String, String> {
    spec.get(&Value::from("nodeSelector"))
        .and_then(Value::as_mapping)
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

fn collect_tolerations(spec: &Mapping) -> Vec<TolerationLite> {
    spec.get(&Value::from("tolerations"))
        .and_then(Value::as_sequence)
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_mapping())
                .map(|m| TolerationLite {
                    key: get_string(m, "key"),
                    operator: get_string(m, "operator"),
                    value: get_string(m, "value"),
                    effect: get_string(m, "effect"),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn detect_required_node_affinity(spec: &Mapping) -> bool {
    spec.get(&Value::from("affinity"))
        .and_then(Value::as_mapping)
        .and_then(|aff| {
            aff.get(&Value::from("nodeAffinity"))
                .and_then(Value::as_mapping)
        })
        .and_then(|na| {
            na.get(&Value::from(
                "requiredDuringSchedulingIgnoredDuringExecution",
            ))
            .and_then(Value::as_mapping)
        })
        .is_some()
}

#[derive(Debug, Clone)]
enum VolumeRef {
    Secret(String),
    ConfigMap(String),
    Other,
}

fn collect_volumes(spec: &Mapping) -> HashMap<String, VolumeRef> {
    let mut map = HashMap::new();
    if let Some(seq) = spec
        .get(&Value::from("volumes"))
        .and_then(Value::as_sequence)
    {
        for v in seq {
            if let Some(m) = v.as_mapping() {
                if let Some(name) = get_string(m, "name") {
                    if let Some(secret) = m.get(&Value::from("secret")).and_then(Value::as_mapping)
                    {
                        if let Some(secret_name) = get_string(secret, "secretName") {
                            map.insert(name.clone(), VolumeRef::Secret(secret_name));
                            continue;
                        }
                        if let Some(secret_name) = get_string(secret, "name") {
                            map.insert(name.clone(), VolumeRef::Secret(secret_name));
                            continue;
                        }
                    }
                    if let Some(cm) = m.get(&Value::from("configMap")).and_then(Value::as_mapping) {
                        if let Some(cm_name) = get_string(cm, "name") {
                            map.insert(name.clone(), VolumeRef::ConfigMap(cm_name));
                            continue;
                        }
                    }
                    map.insert(name, VolumeRef::Other);
                }
            }
        }
    }
    map
}

fn parse_container(
    value: &Value,
    volumes: &HashMap<String, VolumeRef>,
) -> anyhow::Result<ContainerSpecLite> {
    let m = value
        .as_mapping()
        .ok_or_else(|| anyhow!("container entry is not a mapping"))?;

    let name = get_string(m, "name").ok_or_else(|| anyhow!("container missing name"))?;
    let image = get_string(m, "image");

    let (cpu_request_millis, cpu_limit_millis, memory_request_bytes, memory_limit_bytes) =
        parse_resources(m);

    let readiness_probe = m
        .get(&Value::from("readinessProbe"))
        .and_then(Value::as_mapping)
        .map(|p| parse_probe(p, "readiness"));

    let liveness_probe = m
        .get(&Value::from("livenessProbe"))
        .and_then(Value::as_mapping)
        .map(|p| parse_probe(p, "liveness"));

    let startup_probe = m
        .get(&Value::from("startupProbe"))
        .and_then(Value::as_mapping)
        .map(|p| parse_probe(p, "startup"));

    let mut env = BTreeMap::new();
    let mut secret_refs = BTreeSet::new();
    let mut config_map_refs = BTreeSet::new();
    if let Some(env_seq) = m.get(&Value::from("env")).and_then(Value::as_sequence) {
        for e in env_seq {
            if let Some(em) = e.as_mapping() {
                if let Some(env_name) = get_string(em, "name") {
                    let mut entry = EnvValueLite::default();
                    if let Some(val) = get_string(em, "value") {
                        entry.value = Some(val);
                    }
                    if let Some(vf) = em
                        .get(&Value::from("valueFrom"))
                        .and_then(Value::as_mapping)
                    {
                        if let Some(sk) = vf
                            .get(&Value::from("secretKeyRef"))
                            .and_then(Value::as_mapping)
                        {
                            if let Some(secret_name) = get_string(sk, "name") {
                                secret_refs.insert(secret_name.clone());
                                entry.from = Some(EnvFrom::SecretKeyRef {
                                    name: secret_name,
                                    key: get_string(sk, "key"),
                                });
                            }
                        } else if let Some(cm) = vf
                            .get(&Value::from("configMapKeyRef"))
                            .and_then(Value::as_mapping)
                        {
                            if let Some(cm_name) = get_string(cm, "name") {
                                config_map_refs.insert(cm_name.clone());
                                entry.from = Some(EnvFrom::ConfigMapKeyRef {
                                    name: cm_name,
                                    key: get_string(cm, "key"),
                                });
                            }
                        } else if vf.get(&Value::from("fieldRef")).is_some() {
                            entry.from = Some(EnvFrom::FieldRef);
                        } else if vf.get(&Value::from("resourceFieldRef")).is_some() {
                            entry.from = Some(EnvFrom::ResourceFieldRef);
                        }
                    }
                    env.insert(env_name, entry);
                }
            }
        }
    }

    // envFrom references
    if let Some(env_from) = m.get(&Value::from("envFrom")).and_then(Value::as_sequence) {
        for e in env_from {
            if let Some(em) = e.as_mapping() {
                if let Some(sr) = em
                    .get(&Value::from("secretRef"))
                    .and_then(Value::as_mapping)
                {
                    if let Some(name) = get_string(sr, "name") {
                        secret_refs.insert(name);
                    }
                }
                if let Some(cr) = em
                    .get(&Value::from("configMapRef"))
                    .and_then(Value::as_mapping)
                {
                    if let Some(name) = get_string(cr, "name") {
                        config_map_refs.insert(name);
                    }
                }
            }
        }
    }

    // Volume mounts -> capture referenced Secret/ConfigMap names
    if let Some(mounts) = m
        .get(&Value::from("volumeMounts"))
        .and_then(Value::as_sequence)
    {
        for mount in mounts {
            if let Some(mm) = mount.as_mapping() {
                if let Some(vname) = get_string(mm, "name") {
                    if let Some(vref) = volumes.get(&vname) {
                        match vref {
                            VolumeRef::Secret(name) => {
                                secret_refs.insert(name.clone());
                            }
                            VolumeRef::ConfigMap(name) => {
                                config_map_refs.insert(name.clone());
                            }
                            VolumeRef::Other => {}
                        }
                    }
                }
            }
        }
    }

    let ports = m
        .get(&Value::from("ports"))
        .and_then(Value::as_sequence)
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_mapping())
                .map(|pm| ContainerPortLite {
                    container_port: get_i64(pm, "containerPort").map(|v| v as i32),
                    name: get_string(pm, "name"),
                    protocol: get_string(pm, "protocol"),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(ContainerSpecLite {
        name,
        image,
        cpu_request_millis,
        cpu_limit_millis,
        memory_request_bytes,
        memory_limit_bytes,
        readiness_probe,
        liveness_probe,
        startup_probe,
        env,
        ports,
        secret_refs,
        config_map_refs,
    })
}

fn parse_resources(m: &Mapping) -> (Option<i64>, Option<i64>, Option<i64>, Option<i64>) {
    let mut cpu_req = None;
    let mut cpu_lim = None;
    let mut mem_req = None;
    let mut mem_lim = None;

    if let Some(resources) = m.get(&Value::from("resources")).and_then(Value::as_mapping) {
        if let Some(req) = resources
            .get(&Value::from("requests"))
            .and_then(Value::as_mapping)
        {
            cpu_req = req
                .get(&Value::from("cpu"))
                .and_then(Value::as_str)
                .and_then(parse_cpu_to_millis);
            mem_req = req
                .get(&Value::from("memory"))
                .and_then(Value::as_str)
                .and_then(parse_memory_to_bytes);
        }
        if let Some(lim) = resources
            .get(&Value::from("limits"))
            .and_then(Value::as_mapping)
        {
            cpu_lim = lim
                .get(&Value::from("cpu"))
                .and_then(Value::as_str)
                .and_then(parse_cpu_to_millis);
            mem_lim = lim
                .get(&Value::from("memory"))
                .and_then(Value::as_str)
                .and_then(parse_memory_to_bytes);
        }
    }

    (cpu_req, cpu_lim, mem_req, mem_lim)
}

fn parse_probe(p: &Mapping, probe_type: &str) -> ProbeLite {
    let mut probe = ProbeLite {
        probe_type: probe_type.to_string(),
        ..Default::default()
    };

    if let Some(http) = p.get(&Value::from("httpGet")).and_then(Value::as_mapping) {
        probe.probe_type = "http".to_string();
        probe.path = get_string(http, "path");
        probe.port = get_port_string(http.get(&Value::from("port")));
    } else if let Some(tcp) = p.get(&Value::from("tcpSocket")).and_then(Value::as_mapping) {
        probe.probe_type = "tcp".to_string();
        probe.port = get_port_string(tcp.get(&Value::from("port")));
    } else if p.get(&Value::from("exec")).is_some() {
        probe.probe_type = "exec".to_string();
    }

    probe.timeout_seconds = get_i64(p, "timeoutSeconds").map(|v| v as i32);
    probe.period_seconds = get_i64(p, "periodSeconds").map(|v| v as i32);
    probe.failure_threshold = get_i64(p, "failureThreshold").map(|v| v as i32);
    probe.success_threshold = get_i64(p, "successThreshold").map(|v| v as i32);
    probe.initial_delay_seconds = get_i64(p, "initialDelaySeconds").map(|v| v as i32);

    probe
}

fn get_port_string(v: Option<&Value>) -> Option<String> {
    v.and_then(|vv| match vv {
        Value::Number(n) => Some(n.to_string()),
        Value::String(s) => Some(s.clone()),
        _ => None,
    })
}

fn parse_cpu_to_millis(raw: &str) -> Option<i64> {
    if raw.ends_with('m') {
        raw.trim_end_matches('m').parse::<i64>().ok()
    } else {
        raw.parse::<f64>().ok().map(|cores| (cores * 1000.0) as i64)
    }
}

fn parse_memory_to_bytes(raw: &str) -> Option<i64> {
    let raw = raw.trim();
    let units = [
        ("Ki", 1024_f64),
        ("Mi", 1024_f64.powi(2)),
        ("Gi", 1024_f64.powi(3)),
        ("Ti", 1024_f64.powi(4)),
        ("Pi", 1024_f64.powi(5)),
    ];

    for (suffix, mult) in units {
        if raw.ends_with(suffix) {
            let num = raw.trim_end_matches(suffix).parse::<f64>().ok()?;
            return Some((num * mult) as i64);
        }
    }

    if raw.ends_with('k') || raw.ends_with('K') {
        let num = raw
            .trim_end_matches(|c| c == 'k' || c == 'K')
            .parse::<f64>()
            .ok()?;
        return Some((num * 1000.0) as i64);
    }

    if raw.ends_with('M') {
        let num = raw.trim_end_matches('M').parse::<f64>().ok()?;
        return Some((num * 1_000_000.0) as i64);
    }

    raw.parse::<f64>().ok().map(|v| v as i64)
}

fn get_string(m: &Mapping, key: &str) -> Option<String> {
    m.get(&Value::from(key))
        .and_then(Value::as_str)
        .map(|s| s.to_string())
}

fn get_i64(m: &Mapping, key: &str) -> Option<i64> {
    m.get(&Value::from(key)).and_then(|v| match v {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.parse::<i64>().ok(),
        _ => None,
    })
}
