use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::services::storage::index::TemplateIndex;
use crate::templates::manifest::load_and_validate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbedTemplate {
    pub name: String,
    pub path: String,
    pub full_path: PathBuf,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbedPackage {
    pub repo_root: PathBuf,
    pub templates: Vec<ProbedTemplate>,
}

pub fn probe_repo(repo_root: &Path) -> Result<ProbedPackage> {
    let index_path = repo_root.join("index.json");
    let raw = fs::read_to_string(&index_path)
        .with_context(|| format!("failed to read '{}'", index_path.display()))?;
    let index: TemplateIndex = serde_json::from_str(&raw).context("invalid index.json format")?;

    let mut templates = Vec::with_capacity(index.templates.len());
    for entry in index.templates {
        let full_path = repo_root.join(&entry.path);
        if !full_path.is_dir() {
            return Err(anyhow!(
                "template '{}' points to missing directory '{}'",
                entry.name,
                full_path.display()
            ));
        }

        let manifest = load_and_validate(&full_path).map_err(|e| anyhow!(e.to_string()))?;
        templates.push(ProbedTemplate {
            name: entry.name,
            path: entry.path,
            full_path,
            version: manifest.package.version,
            description: manifest.package.description,
        });
    }

    templates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(ProbedPackage {
        repo_root: repo_root.to_path_buf(),
        templates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_manifest(dir: &Path, name: &str, desc: &str) {
        let manifest = format!(
            "[package]\nname = \"{}\"\nversion = \"1.0.0\"\ndescription = \"{}\"\nmin_forge_version = \"0.1.0\"\n",
            name, desc
        );
        fs::write(dir.join("manifest.toml"), manifest).expect("write manifest");
    }

    #[test]
    fn probe_repo_reads_templates_and_manifests() {
        let root = tempfile::tempdir().expect("tempdir");
        let fullstack_dir = root.path().join("templates/fullstack");
        let rust_dir = root.path().join("templates/rust");
        fs::create_dir_all(&fullstack_dir).expect("create fullstack dir");
        fs::create_dir_all(&rust_dir).expect("create rust dir");
        write_manifest(&fullstack_dir, "fullstack", "A fullstack app");
        write_manifest(&rust_dir, "rust", "A rust app");
        fs::write(
            root.path().join("index.json"),
            r#"{"version":1,"templates":[{"name":"rust","path":"templates/rust"},{"name":"fullstack","path":"templates/fullstack"}]}"#,
        )
        .expect("write index");

        let probed = probe_repo(root.path()).expect("probe");
        assert_eq!(probed.templates.len(), 2);
        assert_eq!(probed.templates[0].name, "fullstack");
        assert_eq!(probed.templates[1].name, "rust");
    }

    #[test]
    fn probe_repo_errors_when_index_missing() {
        let root = tempfile::tempdir().expect("tempdir");
        let err = probe_repo(root.path()).expect_err("expected error");
        let msg = format!("{err:#}");
        assert!(msg.contains("failed to read"));
        assert!(msg.contains("index.json"));
    }

    #[test]
    fn probe_repo_errors_when_template_path_missing() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(
            root.path().join("index.json"),
            r#"{"version":1,"templates":[{"name":"fullstack","path":"templates/fullstack"}]}"#,
        )
        .expect("write index");

        let err = probe_repo(root.path()).expect_err("expected error");
        let msg = format!("{err:#}");
        assert!(msg.contains("points to missing directory"));
    }
}
