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

        if !res.findings.is_empty() {
            println!("\nDetected Changes");
            println!("----------------");
            for (i, f) in res.findings.iter().enumerate() {
                let change = arrow_change(f);
                println!(
                    "{}. {}{}",
                    i + 1,
                    f.title,
                    change.map(|c| format!(" ({})", c)).unwrap_or_default()
                );
            }
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
    match (&f.old_value, &f.new_value) {
        (Some(o), Some(n)) => println!("Change: {} \u{2192} {}", o, n),
        (Some(o), None) => println!("Old: {}", o),
        (None, Some(n)) => println!("New: {}", n),
        _ => {}
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

fn arrow_change(f: &Finding) -> Option<String> {
    match (&f.old_value, &f.new_value) {
        (Some(o), Some(n)) => Some(format!("{} \u{2192} {}", o, n)),
        _ => None,
    }
}
