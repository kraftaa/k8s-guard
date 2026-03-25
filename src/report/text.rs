use crate::model::Finding;
use crate::report::ResourceResult;
use std::fmt::Write as _;

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

pub fn render_text_string(results: &[ResourceResult]) -> String {
    let mut buf = String::new();
    for res in results {
        let _ = writeln!(buf, "Resource: {}/{}", res.key.kind, res.key.name);
        if let Some(ns) = &res.key.namespace {
            let _ = writeln!(buf, "Namespace: {}", ns);
        }
        let _ = writeln!(buf, "Overall Rollout Risk: {}", res.overall_risk);

        for note in &res.notes {
            let _ = writeln!(buf, "- {}", note);
        }

        if !res.findings.is_empty() {
            let _ = writeln!(buf, "\nDetected Changes");
            let _ = writeln!(buf, "----------------");
            for (i, f) in res.findings.iter().enumerate() {
                let change = arrow_change(f);
                let _ = writeln!(
                    buf,
                    "{}. {}{}",
                    i + 1,
                    f.title,
                    change.map(|c| format!(" ({})", c)).unwrap_or_default()
                );
            }
        }

        if res.findings.is_empty() {
            let _ = writeln!(buf, "No risky changes detected.");
        } else {
            for f in &res.findings {
                buf.push_str(&format_finding(f));
            }
        }
        buf.push('\n');
    }
    buf
}

fn print_finding(f: &Finding) {
    let prefix = if f
        .container
        .as_ref()
        .map(|c| c.starts_with("init:"))
        .unwrap_or(false)
    {
        "[INIT]"
    } else {
        ""
    };
    println!("\n{}[{}] {} {}", prefix, f.severity, f.rule_id, f.title);
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

fn format_finding(f: &Finding) -> String {
    let mut s = String::new();
    let _ = writeln!(&mut s, "\n[{}] {} {}", f.severity, f.rule_id, f.title);
    if let Some(c) = &f.container {
        let _ = writeln!(&mut s, "Container: {}", c);
    }
    let _ = writeln!(&mut s, "Field: {}", f.field_path);
    match (&f.old_value, &f.new_value) {
        (Some(o), Some(n)) => {
            let _ = writeln!(&mut s, "Change: {} \u{2192} {}", o, n);
        }
        (Some(o), None) => {
            let _ = writeln!(&mut s, "Old: {}", o);
        }
        (None, Some(n)) => {
            let _ = writeln!(&mut s, "New: {}", n);
        }
        _ => {}
    }
    if !f.likely_impact.is_empty() {
        let _ = writeln!(&mut s, "\nLikely impact:");
        for i in &f.likely_impact {
            let _ = writeln!(&mut s, "- {}", i);
        }
    }
    let _ = writeln!(&mut s, "\nWhy it matters:\n{}", f.why_it_matters);
    if !f.suggested_fix.is_empty() {
        let _ = writeln!(&mut s, "\nSuggested fix:");
        for fix in &f.suggested_fix {
            let _ = writeln!(&mut s, "- {}", fix);
        }
    }
    s
}

fn arrow_change(f: &Finding) -> Option<String> {
    match (&f.old_value, &f.new_value) {
        (Some(o), Some(n)) => Some(format!("{} \u{2192} {}", o, n)),
        _ => None,
    }
}
