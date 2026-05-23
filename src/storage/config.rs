use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub user: UserConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UserConfig {
    pub name: String,
    pub email: String,
}

pub struct ConfigStorage {
    config: AppConfig,
    config_file: PathBuf,
}

impl ConfigStorage {
    pub fn new(config_file: &Path) -> Result<Self> {
        let mut storage = Self {
            config: AppConfig::default(),
            config_file: config_file.to_path_buf(),
        };
        storage.load_config()?;
        Ok(storage)
    }

    pub fn load_config(&mut self) -> Result<()> {
        if !self.config_file.exists() {
            return Ok(());
        }

        let raw = fs::read_to_string(&self.config_file)
            .with_context(|| format!("failed to read config '{}'", self.config_file.display()))?;
        self.config = toml::from_str(&raw).context("invalid config.toml")?;
        Ok(())
    }

    pub fn save_config(&self) -> Result<()> {
        let payload = toml::to_string_pretty(&self.config).context("failed to serialize config")?;
        write_atomic(&self.config_file, payload.as_bytes())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.config_file, fs::Permissions::from_mode(0o600))
                .context("failed to set config file permissions")?;
        }

        Ok(())
    }

    pub fn config_path(&self) -> &Path {
        &self.config_file
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    pub fn try_set_value(&mut self, key_path: &str, raw_value: &str) -> Result<()> {
        if key_path.trim().is_empty() {
            return Err(anyhow!("key path cannot be empty"));
        }

        let mut root = Value::try_from(&self.config).context("failed to serialize config")?;
        let keys: Vec<&str> = key_path.split('.').collect();
        if keys.iter().any(|part| part.is_empty()) {
            return Err(anyhow!("invalid key path: {}", key_path));
        }
        let (parents, last) = keys.split_at(keys.len() - 1);

        let mut table = root
            .as_table_mut()
            .ok_or_else(|| anyhow!("config root is not a table"))?;
        for parent in parents {
            table = table
                .get_mut(*parent)
                .and_then(Value::as_table_mut)
                .ok_or_else(|| anyhow!("key path not found: {}", key_path))?;
        }

        let parsed = Self::parse_value(raw_value)?;
        table.insert(last[0].to_string(), parsed);
        self.config = root.try_into().context("failed to update config")?;
        self.save_config()
    }

    pub fn try_get_value(&self, key_path: &str) -> Result<Value> {
        if key_path.trim().is_empty() {
            return Err(anyhow!("key path cannot be empty"));
        }

        let root = Value::try_from(&self.config).context("failed to serialize config")?;
        let mut current = &root;
        for key in key_path.split('.') {
            current = current
                .get(key)
                .ok_or_else(|| anyhow!("key path not found: {}", key_path))?;
        }
        Ok(current.clone())
    }

    pub fn get_flattened_config(&self) -> HashMap<String, String> {
        let root = Value::try_from(&self.config).unwrap_or(Value::Table(Default::default()));
        let mut out = HashMap::new();
        flatten_value(&root, "", &mut out);
        out
    }

    fn parse_value(raw_value: &str) -> Result<Value> {
        if let Ok(parsed) = raw_value.parse::<Value>() {
            return Ok(parsed);
        }
        Ok(Value::String(raw_value.to_string()))
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create '{}'", parent.display()))?;
    }

    let mut tmp = path.to_path_buf();
    tmp.set_extension("tmp");
    fs::write(&tmp, bytes).with_context(|| format!("failed to write '{}'", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| {
        format!(
            "failed to move temporary config '{}' to '{}'",
            tmp.display(),
            path.display()
        )
    })?;
    Ok(())
}

fn flatten_value(value: &Value, prefix: &str, out: &mut HashMap<String, String>) {
    match value {
        Value::String(v) => {
            out.insert(prefix.to_string(), v.clone());
        }
        Value::Integer(v) => {
            out.insert(prefix.to_string(), v.to_string());
        }
        Value::Float(v) => {
            out.insert(prefix.to_string(), v.to_string());
        }
        Value::Boolean(v) => {
            out.insert(prefix.to_string(), v.to_string());
        }
        Value::Table(table) => {
            for (key, val) in table {
                let next = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_value(val, &next, out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_config_file(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("forge-config-test-{name}-{nanos}"))
            .join("config.toml")
    }

    fn cleanup(path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn defaults_when_missing_file() {
        let path = temp_config_file("defaults");
        let storage = ConfigStorage::new(&path).expect("storage");
        assert_eq!(storage.get_config().user.name, "");
        assert_eq!(storage.get_config().user.email, "");
        assert!(!path.exists());
        cleanup(&path);
    }

    #[test]
    fn set_and_get_nested_user_values() {
        let path = temp_config_file("set-get");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        let mut storage = ConfigStorage::new(&path).expect("storage");

        storage
            .try_set_value("user.name", "Alice")
            .expect("set name");
        storage
            .try_set_value("user.email", "alice@example.com")
            .expect("set email");

        assert_eq!(
            storage.try_get_value("user.name").expect("name"),
            Value::String("Alice".to_string())
        );
        assert_eq!(
            storage.try_get_value("user.email").expect("email"),
            Value::String("alice@example.com".to_string())
        );

        cleanup(&path);
    }
}
