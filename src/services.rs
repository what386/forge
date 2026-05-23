use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::lua::{
    PromptConfirmOptions, PromptInputOptions, PromptProvider, PromptSelectOptions, Runtime,
    RuntimeConfig,
};
use crate::templating::manifest::{load_and_validate, Manifest};
use crate::templating::trust::TrustManager;

#[derive(Debug, Clone)]
pub struct PathLayout {
    pub cwd: PathBuf,
    pub local_root: PathBuf,
    pub global_root: PathBuf,
    pub local_templates: PathBuf,
    pub global_templates: PathBuf,
    pub trust_file: PathBuf,
    pub config_file: PathBuf,
}

impl PathLayout {
    pub fn discover(cwd: PathBuf) -> Result<Self> {
        let home = std::env::var_os("HOME").ok_or_else(|| anyhow!("HOME is not set"))?;
        let global_root = PathBuf::from(home).join(".forge");
        Ok(Self {
            cwd: cwd.clone(),
            local_root: cwd.join(".forge"),
            global_root: global_root.clone(),
            local_templates: cwd.join(".forge").join("templates"),
            global_templates: global_root.clone().join("templates"),
            trust_file: global_root.clone().join("trust.json"),
            config_file: global_root.join("config.toml"),
        })
    }
}

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

pub fn run_template(record: &TemplateRecord, project_name: &str, cwd: &Path) -> Result<()> {
    let project_dir = cwd.join(project_name);
    if project_dir.exists() {
        bail!("output directory already exists: {}", project_dir.display());
    }

    let permissions = record
        .manifest
        .requires
        .as_ref()
        .map(|r| r.permissions.clone())
        .unwrap_or_default();

    if !permissions.is_empty() {
        let trust = TrustManager::new(PathLayout::discover(cwd.to_path_buf())?.trust_file);
        let trusted = trust.is_dir_trusted(&record.dir).unwrap_or(false);
        if !trusted {
            eprintln!(
                "Template '{}' requests elevated permissions: {:?}",
                record.name, permissions
            );
            if !confirm_stdin("Proceed? (y/n): ")? {
                bail!("aborted by user");
            }
        }
    }

    fs::create_dir_all(&project_dir)
        .with_context(|| format!("failed to create {}", project_dir.display()))?;

    let main_lua = record.dir.join("main.lua");
    if !main_lua.is_file() {
        bail!("template is missing main.lua");
    }

    let mut runtime = Runtime::new(RuntimeConfig {
        project_name: project_name.to_string(),
        project_dir,
        template_name: record.name.clone(),
        template_dir: record.dir.clone(),
        prompts: Some(std::sync::Arc::new(StdioPrompts {})),
        ..RuntimeConfig::default()
    });

    runtime
        .run(&main_lua.to_string_lossy())
        .map_err(|e| anyhow!(e.to_string()))
}

pub fn validate_template(record: &TemplateRecord) -> Result<Vec<String>> {
    let mut errors = Vec::new();

    let main_lua = record.dir.join("main.lua");
    if !main_lua.is_file() {
        errors.push("main.lua missing".to_string());
        return Ok(errors);
    }

    let source = fs::read_to_string(&main_lua)
        .with_context(|| format!("failed to read {}", main_lua.display()))?;

    let lua = mlua::Lua::new();
    if let Err(e) = lua.load(&source).into_function() {
        errors.push(format!("main.lua syntax error: {}", e));
    }

    for rel in scan_render_refs(&source) {
        let full = record.dir.join("files").join(&rel);
        if !full.is_file() {
            errors.push(format!("referenced file not found: files/{}", rel));
        }
    }

    Ok(errors)
}

fn scan_render_refs(source: &str) -> Vec<String> {
    let re = regex::Regex::new(r#"forge\.(?:render|render_to)\(\s*['\"]([^'\"]+)['\"]"#)
        .expect("valid regex");
    let mut refs = Vec::new();
    for cap in re.captures_iter(source) {
        if let Some(m) = cap.get(1) {
            refs.push(m.as_str().to_string());
        }
    }
    refs
}

fn confirm_stdin(prompt: &str) -> Result<bool> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let ans = line.trim().to_ascii_lowercase();
    Ok(ans == "y" || ans == "yes")
}

struct StdioPrompts;

impl PromptProvider for StdioPrompts {
    fn input(
        &self,
        message: &str,
        opts: PromptInputOptions,
    ) -> Result<String, crate::lua::LuaError> {
        print!("{}", message);
        if !opts.default.is_empty() {
            print!(" [{}]", opts.default);
        }
        print!(": ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(|e| crate::lua::LuaError::new(crate::lua::ErrorKind::Abort, e.to_string()))?;
        let value = line.trim().to_string();
        if value.is_empty() {
            Ok(opts.default)
        } else {
            Ok(value)
        }
    }

    fn confirm(
        &self,
        message: &str,
        opts: PromptConfirmOptions,
    ) -> Result<bool, crate::lua::LuaError> {
        let suffix = if opts.default { "[Y/n]" } else { "[y/N]" };
        print!("{} {}: ", message, suffix);
        let _ = io::stdout().flush();
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(|e| crate::lua::LuaError::new(crate::lua::ErrorKind::Abort, e.to_string()))?;
        let value = line.trim().to_ascii_lowercase();
        if value.is_empty() {
            return Ok(opts.default);
        }
        Ok(matches!(value.as_str(), "y" | "yes"))
    }

    fn select(&self, opts: PromptSelectOptions) -> Result<String, crate::lua::LuaError> {
        println!("{}", opts.message);
        for (idx, opt) in opts.options.iter().enumerate() {
            println!("  {}. {}", idx + 1, opt);
        }
        print!("Choice");
        if !opts.default.is_empty() {
            print!(" [{}]", opts.default);
        }
        print!(": ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(|e| crate::lua::LuaError::new(crate::lua::ErrorKind::Abort, e.to_string()))?;
        let value = line.trim();
        if value.is_empty() && !opts.default.is_empty() {
            return Ok(opts.default);
        }
        if let Ok(i) = value.parse::<usize>() {
            if i >= 1 && i <= opts.options.len() {
                return Ok(opts.options[i - 1].clone());
            }
        }
        if opts.options.iter().any(|o| o == value) {
            return Ok(value.to_string());
        }
        Err(crate::lua::LuaError::new(
            crate::lua::ErrorKind::Abort,
            "invalid selection",
        ))
    }
}
