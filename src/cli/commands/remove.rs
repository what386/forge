use anyhow::{bail, Result};
use std::fs;

use crate::services::paths::PathLayout;
use crate::services::storage::index::TemplateIndexStorage;

pub fn run(names: Vec<String>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;

    let mut existing = Vec::new();
    let mut missing = Vec::new();
    for name in &names {
        let dir = layout.local_templates.join(&name);
        if !dir.exists() {
            missing.push(name.clone());
            continue;
        }
        existing.push(name.clone());
    }

    if !missing.is_empty() {
        missing.sort();
        bail!("local template(s) not found: {}", missing.join(", "));
    }

    for name in &existing {
        let dir = layout.local_templates.join(name);
        fs::remove_dir_all(&dir)?;
    }

    let index = TemplateIndexStorage::new(&layout.local_root);
    index.remove_templates(&existing)?;

    existing.sort();
    for name in existing {
        println!("removed {}", name);
    }
    Ok(())
}
