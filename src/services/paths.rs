use anyhow::{anyhow, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PathLayout {
    pub cwd: PathBuf,
    pub local_root: PathBuf,
    pub global_root: PathBuf,
    pub local_templates: PathBuf,
    pub global_templates: PathBuf,
    pub package_templates: PathBuf,
    pub trust_file: PathBuf,
    pub config_file: PathBuf,
    pub fields_file: PathBuf,
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
            package_templates: global_root.clone().join("packages"),
            trust_file: global_root.clone().join("trust.json"),
            config_file: global_root.clone().join("config.toml"),
            fields_file: global_root.join("fields.json"),
        })
    }
}
