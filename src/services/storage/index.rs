use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemplateIndexEntry {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemplateIndex {
    pub version: u32,
    pub templates: Vec<TemplateIndexEntry>,
}

impl Default for TemplateIndex {
    fn default() -> Self {
        Self {
            version: 1,
            templates: Vec::new(),
        }
    }
}

pub struct TemplateIndexStorage {
    index_file: PathBuf,
}

impl TemplateIndexStorage {
    pub fn new(forge_root: &Path) -> Self {
        Self {
            index_file: forge_root.join("templates.json"),
        }
    }

    pub fn load(&self) -> Result<TemplateIndex> {
        if !self.index_file.exists() {
            return Ok(TemplateIndex::default());
        }

        let raw = fs::read_to_string(&self.index_file)
            .with_context(|| format!("failed to read '{}'", self.index_file.display()))?;
        let index: TemplateIndex =
            serde_json::from_str(&raw).context("invalid templates.json format")?;
        Ok(index)
    }

    pub fn save(&self, index: &TemplateIndex) -> Result<()> {
        if let Some(parent) = self.index_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create '{}'", parent.display()))?;
        }

        let mut normalized = index.clone();
        normalized.version = 1;
        normalized.templates.sort_by(|a, b| a.name.cmp(&b.name));

        let payload = serde_json::to_vec_pretty(&normalized)
            .context("failed to serialize templates index")?;

        let mut tmp = self.index_file.clone();
        tmp.set_extension("tmp");
        fs::write(&tmp, payload).with_context(|| format!("failed to write '{}'", tmp.display()))?;
        fs::rename(&tmp, &self.index_file).with_context(|| {
            format!(
                "failed to move temporary index '{}' to '{}'",
                tmp.display(),
                self.index_file.display()
            )
        })?;
        Ok(())
    }

    pub fn upsert_template(&self, name: &str, path: &str) -> Result<()> {
        let mut by_name: BTreeMap<String, TemplateIndexEntry> = self
            .load()?
            .templates
            .into_iter()
            .map(|entry| (entry.name.clone(), entry))
            .collect();

        by_name.insert(
            name.to_string(),
            TemplateIndexEntry {
                name: name.to_string(),
                path: path.to_string(),
            },
        );

        let index = TemplateIndex {
            version: 1,
            templates: by_name.into_values().collect(),
        };

        self.save(&index)
    }

    pub fn index_path(&self) -> &Path {
        &self.index_file
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("forge-index-test-{name}-{nanos}"))
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn defaults_when_missing_file() {
        let root = temp_root("defaults");
        let storage = TemplateIndexStorage::new(&root);

        let index = storage.load().expect("load");
        assert_eq!(index.version, 1);
        assert!(index.templates.is_empty());
        assert!(!storage.index_path().exists());

        cleanup(&root);
    }

    #[test]
    fn upsert_adds_and_updates_entries() {
        let root = temp_root("upsert");
        let storage = TemplateIndexStorage::new(&root);

        storage
            .upsert_template("rust", "templates/rust")
            .expect("insert rust");
        storage
            .upsert_template("fullstack", "templates/fullstack")
            .expect("insert fullstack");
        storage
            .upsert_template("rust", "templates/rust-v2")
            .expect("update rust");

        let index = storage.load().expect("load index");
        assert_eq!(index.version, 1);
        assert_eq!(index.templates.len(), 2);
        assert_eq!(index.templates[0].name, "fullstack");
        assert_eq!(index.templates[0].path, "templates/fullstack");
        assert_eq!(index.templates[1].name, "rust");
        assert_eq!(index.templates[1].path, "templates/rust-v2");

        cleanup(&root);
    }
}
