use anyhow::{bail, Context, Result};
use std::fs;

use crate::cli::commands::scope;
use crate::services::paths::PathLayout;
use crate::services::storage::config::{ConfigStorage, UserConfig};
use crate::services::storage::index::TemplateIndexStorage;
use crate::templates::ResolveScope;

pub fn run(name: String, local: bool, global: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let scope = scope::resolve(&layout, local, global)?;
    let (root, forge_root) = match scope {
        ResolveScope::Local => (layout.local_templates, layout.local_root),
        ResolveScope::Global => (layout.global_templates, layout.global_root),
    };
    initialize_forge_root(&forge_root)?;
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

    let index = TemplateIndexStorage::new(&forge_root);
    index.upsert_template(&name, &format!("templates/{}", name))?;

    println!("created {}", dir.display());
    Ok(())
}

fn initialize_forge_root(forge_root: &std::path::Path) -> Result<()> {
    fs::create_dir_all(forge_root.join("packages"))
        .with_context(|| format!("failed to create {}", forge_root.join("packages").display()))?;

    let gitignore = forge_root.join(".gitignore");
    let existing = if gitignore.exists() {
        fs::read_to_string(&gitignore)
            .with_context(|| format!("failed to read {}", gitignore.display()))?
    } else {
        String::new()
    };

    let mut contents = existing;
    for entry in ["packages/*", "trust.json"] {
        if !contents.lines().any(|line| line.trim() == entry) {
            if !contents.is_empty() && !contents.ends_with('\n') {
                contents.push('\n');
            }
            contents.push_str(entry);
            contents.push('\n');
        }
    }
    fs::write(&gitignore, contents)
        .with_context(|| format!("failed to write {}", gitignore.display()))?;
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
    fn initialize_forge_root_creates_package_dir_and_ignore_rules() {
        let tmp = tempfile::tempdir().expect("tmp");
        let forge_root = tmp.path().join(".forge");

        initialize_forge_root(&forge_root).expect("initialize");

        assert!(forge_root.join("packages").is_dir());
        assert_eq!(
            fs::read_to_string(forge_root.join(".gitignore")).expect("gitignore"),
            "packages/*\ntrust.json\n"
        );
    }

    #[test]
    fn initialize_forge_root_preserves_existing_ignore_rules() {
        let tmp = tempfile::tempdir().expect("tmp");
        let forge_root = tmp.path().join(".forge");
        fs::create_dir_all(&forge_root).expect("forge root");
        fs::write(forge_root.join(".gitignore"), "index.json\npackages/*\n").expect("write");

        initialize_forge_root(&forge_root).expect("initialize");

        assert_eq!(
            fs::read_to_string(forge_root.join(".gitignore")).expect("gitignore"),
            "index.json\npackages/*\ntrust.json\n"
        );
    }

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
