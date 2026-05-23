use anyhow::{bail, Result};
use std::fs;

use crate::cli::commands::scope;
use crate::services::paths::PathLayout;
use crate::services::storage::index::TemplateIndexStorage;
use crate::templates::ResolveScope;

pub fn run(names: Vec<String>, local: bool, global: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let scope = scope::resolve(&layout, local, global)?;
    let (templates_root, forge_root, missing_label) = match scope {
        ResolveScope::Local => (&layout.local_templates, &layout.local_root, "local"),
        ResolveScope::Global => (&layout.global_templates, &layout.global_root, "global"),
    };

    let mut existing = Vec::new();
    let mut missing = Vec::new();
    for name in &names {
        let dir = templates_root.join(name);
        if !dir.exists() {
            missing.push(name.clone());
            continue;
        }
        existing.push(name.clone());
    }

    if !missing.is_empty() {
        missing.sort();
        bail!(
            "{} template(s) not found: {}",
            missing_label,
            missing.join(", ")
        );
    }

    for name in &existing {
        let dir = templates_root.join(name);
        fs::remove_dir_all(&dir)?;
    }

    let index = TemplateIndexStorage::new(forge_root);
    index.remove_templates(&existing)?;

    existing.sort();
    for name in existing {
        println!("removed {}", name);
    }
    Ok(())
}
