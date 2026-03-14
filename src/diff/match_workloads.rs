use crate::model::{WorkloadKey, WorkloadSpec};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct WorkloadPair {
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
            old: old_item,
            new: Some(nw.clone()),
        });
    }

    // Any resources that existed only in old manifest
    for (_, ow) in old_map {
        pairs.push(WorkloadPair {
            old: Some(ow),
            new: None,
        });
    }

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
