use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::services::paths::PathLayout;
use crate::templates::manifest::{load_and_validate, Manifest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSource {
    Local,
    Global,
    Package,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveScope {
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
            return self.resolve_scoped(name, ResolveScope::Global);
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

        let global_dir = self.layout.global_templates.join(name);
        if global_dir.is_dir() {
            return self.resolve_from_dir(
                name,
                &self.layout.global_templates,
                TemplateSource::Global,
            );
        }

        self.resolve_from_dir(
            name,
            &self.layout.package_templates,
            TemplateSource::Package,
        )
    }

    pub fn resolve_scoped(&self, name: &str, scope: ResolveScope) -> Result<TemplateRecord> {
        match scope {
            ResolveScope::Local => {
                self.resolve_from_dir(name, &self.layout.local_templates, TemplateSource::Local)
            }
            ResolveScope::Global => {
                self.resolve_from_dir(name, &self.layout.global_templates, TemplateSource::Global)
            }
        }
    }

    pub fn resolve_preferred(&self, name: &str, preferred: ResolveScope) -> Result<TemplateRecord> {
        let (base, source) = match preferred {
            ResolveScope::Local => (&self.layout.local_templates, TemplateSource::Local),
            ResolveScope::Global => (&self.layout.global_templates, TemplateSource::Global),
        };
        if base.join(name).is_dir() {
            return self.resolve_from_dir(name, base, source);
        }

        self.resolve_from_dir(
            name,
            &self.layout.package_templates,
            TemplateSource::Package,
        )
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

    pub fn list(
        &self,
        include_local: bool,
        include_global: bool,
        include_package: bool,
    ) -> Result<Vec<TemplateRecord>> {
        let mut out = Vec::new();
        if include_local {
            out.extend(self.scan_root(&self.layout.local_templates, TemplateSource::Local)?);
        }
        if include_global {
            out.extend(self.scan_root(&self.layout.global_templates, TemplateSource::Global)?);
        }
        if include_package {
            out.extend(self.scan_root(&self.layout.package_templates, TemplateSource::Package)?);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(root: &Path) -> PathLayout {
        let cwd = root.join("project");
        let global_root = root.join("home/.forge");
        PathLayout {
            cwd: cwd.clone(),
            local_root: cwd.join(".forge"),
            global_root: global_root.clone(),
            local_templates: cwd.join(".forge/templates"),
            global_templates: global_root.join("templates"),
            package_templates: global_root.join("packages"),
            trust_file: global_root.join("trust.json"),
            config_file: global_root.join("config.toml"),
        }
    }

    fn write_template(dir: &Path, name: &str, description: &str) {
        fs::create_dir_all(dir).expect("create template dir");
        fs::write(dir.join("main.lua"), "-- template").expect("write main");
        fs::write(
            dir.join("manifest.toml"),
            format!(
                "[package]\nname = \"{}\"\nversion = \"1.0.0\"\ndescription = \"{}\"\nmin_forge_version = \"0.1.0\"\n",
                name, description
            ),
        )
        .expect("write manifest");
    }

    #[test]
    fn preferred_resolution_falls_back_to_package_templates() {
        let tmp = tempfile::tempdir().expect("tmp");
        let layout = layout(tmp.path());
        write_template(
            &layout.package_templates.join("webapp"),
            "webapp",
            "Package template",
        );

        let rec = TemplateResolver::new(layout)
            .resolve_preferred("webapp", ResolveScope::Local)
            .expect("resolve package fallback");

        assert_eq!(rec.source, TemplateSource::Package);
        assert_eq!(rec.manifest.package.description, "Package template");
    }

    #[test]
    fn explicit_scoped_resolution_does_not_fall_back_to_package_templates() {
        let tmp = tempfile::tempdir().expect("tmp");
        let layout = layout(tmp.path());
        write_template(
            &layout.package_templates.join("webapp"),
            "webapp",
            "Package template",
        );

        let err = TemplateResolver::new(layout)
            .resolve_scoped("webapp", ResolveScope::Local)
            .expect_err("local should not fall back to package");

        assert!(format!("{err:#}").contains("template 'webapp' not found"));
    }

    #[test]
    fn list_includes_package_templates_when_requested() {
        let tmp = tempfile::tempdir().expect("tmp");
        let layout = layout(tmp.path());
        write_template(
            &layout.local_templates.join("local-template"),
            "local-template",
            "Local template",
        );
        write_template(
            &layout.package_templates.join("package-template"),
            "package-template",
            "Package template",
        );

        let records = TemplateResolver::new(layout)
            .list(true, false, true)
            .expect("list templates");
        let sources: Vec<_> = records.iter().map(|rec| rec.source).collect();

        assert_eq!(records.len(), 2);
        assert!(sources.contains(&TemplateSource::Local));
        assert!(sources.contains(&TemplateSource::Package));
    }
}
