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
    let risk = v["resources"][0]["overall_risk"].as_str().unwrap_or("SAFE");
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

#[test]
fn notes_new_and_removed_resources() {
    // new resource only in new manifest
    let mut cmd_new = Command::cargo_bin("k8s-diff-explainer").unwrap();
    let out_new = cmd_new
        .arg(fixture("old.yaml")) // baseline
        .arg(fixture("only-new.yaml"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout_new = String::from_utf8(out_new).unwrap();
    assert!(stdout_new.contains("New resource detected: StatefulSet/fresh"));

    // resource only in old manifest
    let mut cmd_old = Command::cargo_bin("k8s-diff-explainer").unwrap();
    let out_old = cmd_old
        .arg(fixture("only-old.yaml"))
        .arg(fixture("new.yaml")) // new manifest
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout_old = String::from_utf8(out_old).unwrap();
    assert!(stdout_old.contains("Resource removed in new manifest: Deployment/orphan"));
}

#[test]
fn experimental_rule_only_when_flagged() {
    // selector changed should only fire with --experimental
    let mut cmd_default = Command::cargo_bin("k8s-diff-explainer").unwrap();
    let out_default = cmd_default
        .arg(fixture("selector-old.yaml"))
        .arg(fixture("selector-new.yaml"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout_default = String::from_utf8(out_default).unwrap();
    assert!(
        !stdout_default.contains("KDX011"),
        "experimental rule should be off by default"
    );

    let mut cmd_exp = Command::cargo_bin("k8s-diff-explainer").unwrap();
    let out_exp = cmd_exp
        .arg(fixture("selector-old.yaml"))
        .arg(fixture("selector-new.yaml"))
        .arg("--experimental")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout_exp = String::from_utf8(out_exp).unwrap();
    assert!(
        stdout_exp.contains("KDX011"),
        "experimental rule should fire when flag is set"
    );
}

#[test]
fn experimental_rule_catches_match_expressions() {
    let mut cmd = Command::cargo_bin("k8s-diff-explainer").unwrap();
    let output = cmd
        .arg(fixture("selector-expr-old.yaml"))
        .arg(fixture("selector-expr-new.yaml"))
        .arg("--experimental")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    assert!(
        stdout.contains("KDX011"),
        "should flag selector drift when matchExpressions change"
    );
}

#[test]
fn init_containers_are_checked() {
    let mut cmd = Command::cargo_bin("k8s-diff-explainer").unwrap();
    let output = cmd
        .arg(fixture("init-old.yaml"))
        .arg(fixture("init-new.yaml"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    assert!(
        stdout.contains("init:migrate"),
        "init container should be labeled in output"
    );
    assert!(
        stdout.contains("Memory limit reduced"),
        "init container diff should trigger rules"
    );
}

#[test]
fn summary_only_text() {
    let mut cmd = Command::cargo_bin("k8s-diff-explainer").unwrap();
    let output = cmd
        .arg(fixture("old.yaml"))
        .arg(fixture("new.yaml"))
        .arg("--summary-only")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("Summary: Overall"));
    assert!(
        !stdout.contains("Resource:"),
        "summary-only should not print full report"
    );
}

#[test]
fn summary_only_json() {
    let mut cmd = Command::cargo_bin("k8s-diff-explainer").unwrap();
    let output = cmd
        .arg(fixture("old.yaml"))
        .arg(fixture("new.yaml"))
        .arg("--format")
        .arg("json")
        .arg("--summary-only")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("Summary: Overall"));
    assert!(
        !stdout.contains('{'),
        "summary-only should suppress JSON output on stdout"
    );
}
