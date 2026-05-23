use anyhow::{bail, Context, Result};
use std::fs;

use crate::storage::config::{ConfigStorage, UserConfig};
use crate::storage::paths::PathLayout;

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

    let profile = ConfigStorage::new(&layout.config_file)?
        .get_config()
        .user
        .clone();
    fs::write(
        dir.join("manifest.toml"),
        manifest_contents(&name, env!("CARGO_PKG_VERSION"), &profile),
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

fn manifest_contents(name: &str, forge_version: &str, user: &UserConfig) -> String {
    let mut out = format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\ndescription = \"TODO\"\nmin_forge_version = \"{}\"\n",
        toml_escape(name),
        toml_escape(forge_version)
    );
    let has_name = !user.name.is_empty();
    let has_email = !user.email.is_empty();
    if has_name || has_email {
        out.push_str("\n[author]\n");
        if has_name {
            out.push_str(&format!("name = \"{}\"\n", toml_escape(&user.name)));
        }
        if has_email {
            out.push_str(&format!("email = \"{}\"\n", toml_escape(&user.email)));
        }
    }
    out
}

fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_contents_without_profile() {
        let out = manifest_contents("webapp", "1.2.3", &UserConfig::default());
        assert!(out.contains("[package]"));
        assert!(!out.contains("[author]"));
    }

    #[test]
    fn manifest_contents_with_profile() {
        let profile = UserConfig {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        let out = manifest_contents("webapp", "1.2.3", &profile);
        assert!(out.contains("[author]"));
        assert!(out.contains("name = \"Test User\""));
        assert!(out.contains("email = \"test@example.com\""));
    }
}
