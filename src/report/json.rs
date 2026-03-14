use crate::report::ResourceResult;
use anyhow::Result;

pub fn render_json(results: &[ResourceResult]) -> Result<()> {
    let payload = serde_json::json!({ "resources": results });
    let out = serde_json::to_string_pretty(&payload)?;
    println!("{}", out);
    Ok(())
}
