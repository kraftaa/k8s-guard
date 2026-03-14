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
use report::{ResourceResult, render_json, render_text};
use rules::{run_rules, score_findings};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let old_specs = load_workloads(&cli.old)?;
    let new_specs = load_workloads(&cli.new)?;

    let pairs = match_workloads(&old_specs, &new_specs);
    let mut results = Vec::new();

    for pair in pairs {
        if let Some(new_spec) = pair.new {
            match pair.old {
                Some(old_spec) => {
                    let findings = run_rules(&old_spec, &new_spec);
                    let overall_risk = score_findings(&findings);
                    results.push(ResourceResult {
                        key: new_spec.key.clone(),
                        findings,
                        overall_risk,
                        notes: Vec::new(),
                    });
                }
                None => {
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
            }
        }
    }

    match cli.format {
        OutputFormat::Text => render_text(&results),
        OutputFormat::Json => render_json(&results)?,
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
