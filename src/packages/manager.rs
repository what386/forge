use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::packages::probe::ProbedPackage;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageRecord {
    pub repo: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub installed_at: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageIndex {
    pub version: u32,
    pub packages: BTreeMap<String, PackageRecord>,
}

impl Default for PackageIndex {
    fn default() -> Self {
        Self {
            version: 1,
            packages: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledTemplate {
    pub name: String,
    pub destination: PathBuf,
    pub checksum: String,
}

pub struct PackageManager {
    forge_root: PathBuf,
    packages_root: PathBuf,
    index_path: PathBuf,
}

impl PackageManager {
    pub fn global() -> Result<Self> {
        let home = std::env::var_os("HOME").ok_or_else(|| anyhow!("HOME is not set"))?;
        let forge_root = PathBuf::from(home).join(".forge");
        Ok(Self {
            packages_root: forge_root.join("packages"),
            index_path: forge_root.join("packages.json"),
            forge_root,
        })
    }

    pub fn list(&self) -> Result<Vec<(String, PackageRecord)>> {
        let index = self.load_index()?;
        Ok(index.packages.into_iter().collect())
    }

    pub fn load_index(&self) -> Result<PackageIndex> {
        if !self.index_path.exists() {
            return Ok(PackageIndex::default());
        }
        let raw = fs::read_to_string(&self.index_path)
            .with_context(|| format!("failed to read '{}'", self.index_path.display()))?;
        let index: PackageIndex =
            serde_json::from_str(&raw).context("invalid packages.json format")?;
        Ok(index)
    }

    pub fn install_templates(
        &self,
        repo: &str,
        probed: &ProbedPackage,
        template_names: &[String],
    ) -> Result<Vec<InstalledTemplate>> {
        if template_names.is_empty() {
            bail!("template selection cannot be empty");
        }
        fs::create_dir_all(&self.packages_root)
            .with_context(|| format!("failed to create '{}'", self.packages_root.display()))?;

        let mut index = self.load_index()?;
        let requested: BTreeSet<&str> = template_names.iter().map(|n| n.as_str()).collect();
        let mut selected = Vec::new();
        for template in &probed.templates {
            if requested.contains(template.name.as_str()) {
                selected.push(template);
            }
        }

        let selected_names: BTreeSet<&str> = selected.iter().map(|t| t.name.as_str()).collect();
        let mut missing = Vec::new();
        for name in requested {
            if !selected_names.contains(name) {
                missing.push(name.to_string());
            }
        }
        if !missing.is_empty() {
            missing.sort();
            bail!("unknown template selection: {}", missing.join(", "));
        }

        let mut installed = Vec::with_capacity(selected.len());
        for template in selected {
            if let Some(existing) = index.packages.get(&template.name) {
                if existing.repo != repo {
                    bail!(
                        "template '{}' is already installed from '{}'",
                        template.name,
                        existing.repo
                    );
                }
            }

            let destination = self.packages_root.join(&template.name);
            if destination.exists() {
                fs::remove_dir_all(&destination)
                    .with_context(|| format!("failed to remove '{}'", destination.display()))?;
            }
            copy_dir_recursive(&template.full_path, &destination)?;

            let checksum = checksum_dir(&destination)?;
            let record = PackageRecord {
                repo: repo.to_string(),
                ref_name: "main".to_string(),
                installed_at: now_rfc3339()?,
                checksum: checksum.clone(),
            };
            index.packages.insert(template.name.clone(), record);
            installed.push(InstalledTemplate {
                name: template.name.clone(),
                destination,
                checksum,
            });
        }
        self.save_index(&index)?;
        installed.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(installed)
    }

    pub fn remove_templates(&self, names: &[String]) -> Result<Vec<String>> {
        let mut index = self.load_index()?;
        let mut removed = Vec::new();
        for name in names {
            if index.packages.remove(name).is_some() {
                let dir = self.packages_root.join(name);
                if dir.exists() {
                    fs::remove_dir_all(&dir)
                        .with_context(|| format!("failed to remove '{}'", dir.display()))?;
                }
                removed.push(name.clone());
            }
        }
        self.save_index(&index)?;
        removed.sort();
        Ok(removed)
    }

    pub fn remove_all(&self) -> Result<Vec<String>> {
        let names: Vec<String> = self.load_index()?.packages.into_keys().collect();
        self.remove_templates(&names)
    }

    fn save_index(&self, index: &PackageIndex) -> Result<()> {
        fs::create_dir_all(&self.forge_root)
            .with_context(|| format!("failed to create '{}'", self.forge_root.display()))?;
        let mut normalized = index.clone();
        normalized.version = 1;
        let payload =
            serde_json::to_vec_pretty(&normalized).context("failed to serialize packages index")?;
        let mut tmp = self.index_path.clone();
        tmp.set_extension("tmp");
        fs::write(&tmp, payload).with_context(|| format!("failed to write '{}'", tmp.display()))?;
        fs::rename(&tmp, &self.index_path).with_context(|| {
            format!(
                "failed to move temporary index '{}' to '{}'",
                tmp.display(),
                self.index_path.display()
            )
        })?;
        Ok(())
    }
}

fn now_rfc3339() -> Result<String> {
    let now = OffsetDateTime::from(SystemTime::now());
    now.format(&Rfc3339)
        .map_err(|e| anyhow!("failed to format timestamp: {}", e))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).with_context(|| format!("failed to create '{}'", dst.display()))?;
    for entry in fs::read_dir(src).with_context(|| format!("failed to read '{}'", src.display()))? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            fs::copy(&path, &target).with_context(|| {
                format!(
                    "failed to copy file '{}' to '{}'",
                    path.display(),
                    target.display()
                )
            })?;
        }
    }
    Ok(())
}

fn checksum_dir(dir: &Path) -> Result<String> {
    if !dir.is_dir() {
        bail!("expected directory for checksum: {}", dir.display());
    }
    let mut files = Vec::new();
    collect_files(dir, dir, &mut files)?;
    files.sort();

    let mut hasher = Sha256::new();
    for rel in files {
        let full = dir.join(&rel);
        hasher.update(rel.as_bytes());
        hasher.update([0u8]);
        hasher.update(
            fs::read(&full)
                .with_context(|| format!("failed to read file '{}' for checksum", full.display()))?,
        );
        hasher.update([0u8]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn collect_files(root: &Path, current: &Path, out: &mut Vec<String>) -> Result<()> {
    for entry in
        fs::read_dir(current).with_context(|| format!("failed to read '{}'", current.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let ft = entry
            .file_type()
            .with_context(|| format!("failed to stat '{}'", path.display()))?;
        if ft.is_dir() {
            collect_files(root, &path, out)?;
            continue;
        }
        if ft.is_symlink() {
            bail!("symlinks are not supported in package templates: {}", path.display());
        }
        if ft.is_file() {
            let rel = path
                .strip_prefix(root)
                .map_err(|e| anyhow!("failed to strip prefix: {}", e))?;
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packages::probe::ProbedTemplate;

    fn make_template(root: &Path, name: &str) -> ProbedTemplate {
        let path = root.join(name);
        fs::create_dir_all(&path).expect("mkdir");
        fs::write(path.join("main.lua"), "-- lua").expect("write");
        ProbedTemplate {
            name: name.to_string(),
            path: name.to_string(),
            full_path: path,
            version: "1.0.0".to_string(),
            description: "desc".to_string(),
        }
    }

    #[test]
    fn checksum_changes_when_file_changes() {
        let tmp = tempfile::tempdir().expect("tmp");
        let dir = tmp.path().join("pkg");
        fs::create_dir_all(&dir).expect("mkdir");
        fs::write(dir.join("a.txt"), "one").expect("write");
        let c1 = checksum_dir(&dir).expect("checksum1");
        fs::write(dir.join("a.txt"), "two").expect("write2");
        let c2 = checksum_dir(&dir).expect("checksum2");
        assert_ne!(c1, c2);
    }

    #[test]
    fn install_templates_rejects_conflicting_repo() {
        let home = tempfile::tempdir().expect("home");
        std::env::set_var("HOME", home.path());
        let manager = PackageManager::global().expect("manager");
        let repo = tempfile::tempdir().expect("repo");
        let probed = ProbedPackage {
            repo_root: repo.path().to_path_buf(),
            templates: vec![make_template(repo.path(), "fullstack")],
        };
        manager
            .install_templates(
                "https://example.com/a.git",
                &probed,
                &[String::from("fullstack")],
            )
            .expect("first install");
        let err = manager
            .install_templates(
                "https://example.com/b.git",
                &probed,
                &[String::from("fullstack")],
            )
            .expect_err("conflict");
        assert!(format!("{err:#}").contains("already installed"));
    }
}
