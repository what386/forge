use anyhow::{Context, Result};
use regex::Regex;

use crate::lua::fs::safe_template_path;
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

    for reference in scan_static_refs(&source) {
        match safe_template_path(&record.dir.join("files"), &reference.path) {
            Ok(full) => {
                if reference.kind == RefKind::Directory && !full.is_dir() {
                    errors.push(format!(
                        "referenced directory not found: files/{}",
                        reference.path
                    ));
                } else if reference.kind == RefKind::File && !full.is_file() {
                    errors.push(format!(
                        "referenced file not found: files/{}",
                        reference.path
                    ));
                }
            }
            Err(e) => errors.push(format!(
                "invalid template file reference files/{}: {}",
                reference.path, e
            )),
        }
    }

    Ok(errors)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticRef {
    path: String,
    kind: RefKind,
}

fn scan_static_refs(source: &str) -> Vec<StaticRef> {
    let re = Regex::new(
        r#"forge\.(render|render_to|render_dir|render_dir_to)\(\s*['\"]([^'\"]+)['\"]|forge\.fs\.add\(\s*['\"]([^'\"]+)['\"]"#,
    )
    .expect("valid regex");
    let mut refs = Vec::new();
    for cap in re.captures_iter(source) {
        if let Some(method) = cap.get(1) {
            let kind = match method.as_str() {
                "render_dir" | "render_dir_to" => RefKind::Directory,
                _ => RefKind::File,
            };
            if let Some(path) = cap.get(2) {
                refs.push(StaticRef {
                    path: path.as_str().to_string(),
                    kind,
                });
            }
            continue;
        }
        if let Some(path) = cap.get(3) {
            refs.push(StaticRef {
                path: path.as_str().to_string(),
                kind: RefKind::File,
            });
        }
    }
    refs
}
