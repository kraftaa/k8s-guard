use crate::model::{Finding, WorkloadSpec};

pub trait Rule {
    fn id(&self) -> &'static str;
    fn check(&self, old: &WorkloadSpec, new: &WorkloadSpec) -> Vec<Finding>;
}
