use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::services::paths::PathLayout;
use crate::templates::manifest::{load_and_validate, Manifest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSource {
    Local,
    Global,
}

#[derive(Debug, Clone)]
pub struct TemplateRecord {
    pub name: String,
    pub dir: PathBuf,
    pub source: TemplateSource,
    pub manifest: Manifest,
}

pub struct TemplateResolver {
    layout: PathLayout,
}

impl TemplateResolver {
    pub fn new(layout: PathLayout) -> Self {
        Self { layout }
    }

    pub fn resolve(&self, name: &str, force_global: bool) -> Result<TemplateRecord> {
        if force_global {
            return self.resolve_from_dir(
                name,
                &self.layout.global_templates,
                TemplateSource::Global,
            );
        }

        let local_dir = self.layout.local_templates.join(name);
        if local_dir.is_dir() {
            let manifest = load_and_validate(&local_dir).map_err(|e| anyhow!(e.to_string()))?;
            return Ok(TemplateRecord {
                name: name.to_string(),
                dir: local_dir,
                source: TemplateSource::Local,
                manifest,
            });
        }

        self.resolve_from_dir(name, &self.layout.global_templates, TemplateSource::Global)
    }

    fn resolve_from_dir(
        &self,
        name: &str,
        base: &Path,
        source: TemplateSource,
    ) -> Result<TemplateRecord> {
        let dir = base.join(name);
        if !dir.is_dir() {
            bail!("template '{}' not found", name);
        }
        let manifest = load_and_validate(&dir).map_err(|e| anyhow!(e.to_string()))?;
        Ok(TemplateRecord {
            name: name.to_string(),
            dir,
            source,
            manifest,
        })
    }

    pub fn list(&self, include_local: bool, include_global: bool) -> Result<Vec<TemplateRecord>> {
        let mut out = Vec::new();
        if include_local {
            out.extend(self.scan_root(&self.layout.local_templates, TemplateSource::Local)?);
        }
        if include_global {
            out.extend(self.scan_root(&self.layout.global_templates, TemplateSource::Global)?);
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }

    fn scan_root(&self, root: &Path, source: TemplateSource) -> Result<Vec<TemplateRecord>> {
        if !root.is_dir() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in
            fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if let Ok(manifest) = load_and_validate(&path) {
                out.push(TemplateRecord {
                    name: name.to_string(),
                    dir: path,
                    source,
                    manifest,
                });
            }
        }
        Ok(out)
    }
}
