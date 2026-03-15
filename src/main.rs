mod cli;
mod diff;
mod manifest;
mod model;
mod report;
mod rules;

use clap::Parser;
use cli::{Cli, FailThreshold, OutputFormat};
use diff::match_workloads;
use manifest::load_workloads;
use model::OverallRisk;
use report::{ResourceResult, render_json, render_json_string, render_text, render_text_string};
use rules::{run_rules, score_findings};
use std::fs;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let old_specs = load_workloads(&cli.old)?;
    let new_specs = load_workloads(&cli.new)?;

    let pairs = match_workloads(&old_specs, &new_specs);
    let mut results = Vec::new();

    for pair in pairs {
        match (pair.old, pair.new) {
            (Some(old_spec), Some(new_spec)) => {
                let findings = run_rules(&old_spec, &new_spec, cli.experimental);
                let overall_risk = score_findings(&findings);
                results.push(ResourceResult {
                    key: new_spec.key.clone(),
                    findings,
                    overall_risk,
                    notes: Vec::new(),
                });
            }
            (None, Some(new_spec)) => {
                results.push(ResourceResult {
                    key: new_spec.key.clone(),
                    findings: Vec::new(),
                    overall_risk: OverallRisk::Safe,
                    notes: vec![format!(
                        "New resource detected: {}/{}; no baseline to diff.",
                        new_spec.key.kind, new_spec.key.name
                    )],
                });
            }
            (Some(old_spec), None) => {
                results.push(ResourceResult {
                    key: old_spec.key.clone(),
                    findings: Vec::new(),
                    overall_risk: OverallRisk::Safe,
                    notes: vec![format!(
                        "Resource removed in new manifest: {}/{}; risk not evaluated.",
                        old_spec.key.kind, old_spec.key.name
                    )],
                });
            }
            (None, None) => {}
        }
    }

    if let Some(path) = &cli.output {
        match cli.format {
            OutputFormat::Text => {
                let body = render_text_string(&results);
                fs::write(path, body)?;
            }
            OutputFormat::Json => {
                let body = render_json_string(&results)?;
                fs::write(path, body)?;
            }
        }
    }

    if !cli.summary_only {
        match cli.format {
            OutputFormat::Text => {
                render_text(&results);
            }
            OutputFormat::Json => {
                if cli.output.is_none() {
                    render_json(&results)?;
                }
            }
        }
    }

    if cli.summary_only || matches!(cli.format, OutputFormat::Text) || cli.output.is_some() {
        print_summary(&results);
    }

    if let Some(threshold) = cli.fail_on {
        let should_fail = results
            .iter()
            .any(|r| meets_threshold(r.overall_risk, threshold));
        if should_fail {
            std::process::exit(1);
        }
    }

    Ok(())
}

fn meets_threshold(risk: OverallRisk, threshold: FailThreshold) -> bool {
    match threshold {
        FailThreshold::Medium => matches!(risk, OverallRisk::Medium | OverallRisk::High),
        FailThreshold::High => matches!(risk, OverallRisk::High),
    }
}

fn print_summary(results: &[ResourceResult]) {
    let worst = results
        .iter()
        .map(|r| &r.overall_risk)
        .max_by_key(|r| match r {
            OverallRisk::Safe => 0,
            OverallRisk::Low => 1,
            OverallRisk::Medium => 2,
            OverallRisk::High => 3,
        })
        .unwrap_or(&OverallRisk::Safe);
    let total_findings: usize = results.iter().map(|r| r.findings.len()).sum();
    let total_high: usize = results
        .iter()
        .flat_map(|r| &r.findings)
        .filter(|f| matches!(f.severity, model::Severity::High))
        .count();
    println!(
        "Summary: Overall {} ({} findings, {} high) across {} resource(s)",
        worst,
        total_findings,
        total_high,
        results.len()
    );
}
