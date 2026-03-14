use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

#[test]
fn text_output_reports_high_risk() {
    let mut cmd = Command::cargo_bin("k8s-diff-explainer").unwrap();
    let assert = cmd
        .arg(fixture("old.yaml"))
        .arg(fixture("new.yaml"))
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Overall Rollout Risk: HIGH"));
    assert!(stdout.contains("KDX001"));
    assert!(stdout.contains("Memory limit reduced"));
    assert!(stdout.contains("Replica increase with pressure signals"));
}

#[test]
fn json_output_is_high() {
    let mut cmd = Command::cargo_bin("k8s-diff-explainer").unwrap();
    let output = cmd
        .arg(fixture("old.yaml"))
        .arg(fixture("new.yaml"))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    let risk = v["resources"][0]["overall_risk"]
        .as_str()
        .unwrap_or("SAFE");
    assert_eq!(risk, "HIGH");
}

#[test]
fn fail_on_high_exits_nonzero() {
    let mut cmd = Command::cargo_bin("k8s-diff-explainer").unwrap();
    cmd.arg(fixture("old.yaml"))
        .arg(fixture("new.yaml"))
        .arg("--fail-on")
        .arg("high")
        .assert()
        .failure();
}
