use anyhow::{bail, Context, Result};
use std::fs;

use crate::services::PathLayout;

pub fn run(name: String, global: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let root = if global {
        layout.global_templates
    } else {
        layout.local_templates
    };
    let dir = root.join(&name);
    if dir.exists() {
        bail!("template already exists: {}", dir.display());
    }

    fs::create_dir_all(dir.join("files"))
        .with_context(|| format!("failed to create {}", dir.display()))?;

    fs::write(
        dir.join("manifest.toml"),
        format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\ndescription = \"TODO\"\nmin_forge_version = \"{}\"\n",
            name,
            env!("CARGO_PKG_VERSION")
        ),
    )?;

    fs::write(
        dir.join("main.lua"),
        "-- Minimal template\nforge.render(\"README.md.tpl\")\n",
    )?;

    fs::write(
        dir.join("files").join("README.md.tpl"),
        "# {{ forge.project.name }}\n",
    )?;

    println!("created {}", dir.display());
    Ok(())
}
