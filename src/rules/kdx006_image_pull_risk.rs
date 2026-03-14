use crate::model::{Confidence, Severity, WorkloadSpec};
use crate::rules::traits::Rule;
use crate::rules::{base_finding, format_field_path, pair_containers};

pub struct ImagePullRiskRule;

impl Rule for ImagePullRiskRule {
    fn id(&self) -> &'static str {
        "KDX006"
    }

    fn check(&self, old: &WorkloadSpec, new: &WorkloadSpec) -> Vec<crate::model::Finding> {
        let mut findings = Vec::new();
        for (oc, nc) in pair_containers(old, new) {
            if let (Some(oi), Some(ni)) = (&oc.image, &nc.image) {
                if oi == ni {
                    continue;
                }
                let mut suspicious = false;
                let mut severity = Severity::Medium;

                if ni.trim().is_empty() || ni.ends_with(':') {
                    suspicious = true;
                    severity = Severity::High;
                }

                let old_reg = registry_prefix(oi);
                let new_reg = registry_prefix(ni);
                if old_reg != new_reg && new_reg.is_some() {
                    if new.image_pull_secrets.is_empty() {
                        severity = Severity::High;
                    }
                    suspicious = true;
                }

                if ni.ends_with(":latest") {
                    suspicious = true;
                }

                if suspicious {
                    findings.push(base_finding(
                        self.id(),
                        severity,
                        Confidence::Medium,
                        old,
                        new,
                        Some(nc.name.clone()),
                        format_field_path(&nc.name, "image"),
                        "Image pull risk introduced",
                        Some(oi.clone()),
                        Some(ni.clone()),
                        vec!["ErrImagePull / ImagePullBackOff", "Rollout blocked before start"],
                        "Image/registry change without clear tag or pull secret can prevent pods from starting.",
                        vec![
                            "verify new image name and tag",
                            "ensure imagePullSecrets are set for private registry",
                            "avoid empty or implicit tags in CI",
                        ],
                    ));
                }
            }
        }
        findings
    }
}

fn registry_prefix(image: &str) -> Option<String> {
    let parts: Vec<&str> = image.split('/').collect();
    if parts.len() > 1 && parts[0].contains('.') {
        Some(parts[0].to_string())
    } else {
        None
    }
}
