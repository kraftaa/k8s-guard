use crate::model::Finding;
use crate::report::ResourceResult;

pub fn render_text(results: &[ResourceResult]) {
    for res in results {
        println!("Resource: {}/{}", res.key.kind, res.key.name);
        if let Some(ns) = &res.key.namespace {
            println!("Namespace: {}", ns);
        }
        println!("Overall Rollout Risk: {}", res.overall_risk);

        for note in &res.notes {
            println!("- {}", note);
        }

        if res.findings.is_empty() {
            println!("No risky changes detected.");
        } else {
            for f in &res.findings {
                print_finding(f);
            }
        }
        println!();
    }
}

fn print_finding(f: &Finding) {
    println!("\n[{}] {} {}", f.severity, f.rule_id, f.title);
    if let Some(c) = &f.container {
        println!("Container: {}", c);
    }
    println!("Field: {}", f.field_path);
    if let Some(old) = &f.old_value {
        println!("Old: {}", old);
    }
    if let Some(new) = &f.new_value {
        println!("New: {}", new);
    }
    if !f.likely_impact.is_empty() {
        println!("\nLikely impact:");
        for i in &f.likely_impact {
            println!("- {}", i);
        }
    }
    println!("\nWhy it matters:\n{}", f.why_it_matters);
    if !f.suggested_fix.is_empty() {
        println!("\nSuggested fix:");
        for s in &f.suggested_fix {
            println!("- {}", s);
        }
    }
}
