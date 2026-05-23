use anyhow::{Context, Result};
use regex::Regex;

use crate::templates::resolver::TemplateRecord;

pub fn validate_template(record: &TemplateRecord) -> Result<Vec<String>> {
    let mut errors = Vec::new();

    let main_lua = record.dir.join("main.lua");
    if !main_lua.is_file() {
        errors.push("main.lua missing".to_string());
        return Ok(errors);
    }

    let source = std::fs::read_to_string(&main_lua)
        .with_context(|| format!("failed to read {}", main_lua.display()))?;

    let lua = mlua::Lua::new();
    if let Err(e) = lua.load(&source).into_function() {
        errors.push(format!("main.lua syntax error: {}", e));
    }

    for rel in scan_render_refs(&source) {
        let full = record.dir.join("files").join(&rel);
        if !full.is_file() {
            errors.push(format!("referenced file not found: files/{}", rel));
        }
    }

    Ok(errors)
}

fn scan_render_refs(source: &str) -> Vec<String> {
    let re =
        Regex::new(r#"forge\.(?:render|render_to)\(\s*['\"]([^'\"]+)['\"]"#).expect("valid regex");
    let mut refs = Vec::new();
    for cap in re.captures_iter(source) {
        if let Some(m) = cap.get(1) {
            refs.push(m.as_str().to_string());
        }
    }
    refs
}
