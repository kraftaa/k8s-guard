use crate::model::{WorkloadKey, WorkloadSpec};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct WorkloadPair {
    pub key: WorkloadKey,
    pub old: Option<WorkloadSpec>,
    pub new: Option<WorkloadSpec>,
}

pub fn match_workloads(old: &[WorkloadSpec], new: &[WorkloadSpec]) -> Vec<WorkloadPair> {
    let mut old_map: BTreeMap<String, WorkloadSpec> = BTreeMap::new();
    for w in old {
        old_map.insert(key_string(&w.key), w.clone());
    }

    let mut pairs = Vec::new();
    for nw in new {
        let k = key_string(&nw.key);
        let old_item = old_map.remove(&k);
        pairs.push(WorkloadPair {
            key: nw.key.clone(),
            old: old_item,
            new: Some(nw.clone()),
        });
    }

    // Resources only in old manifest are currently ignored for rollout-risk
    pairs
}

fn key_string(key: &WorkloadKey) -> String {
    format!(
        "{}:{}:{}",
        key.kind,
        key.namespace.clone().unwrap_or_default(),
        key.name
    )
}
