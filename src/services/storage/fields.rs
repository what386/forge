use anyhow::{anyhow, Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FieldsStorage {
    fields_file: PathBuf,
    fields: BTreeMap<String, String>,
}

impl FieldsStorage {
    pub fn new(fields_file: &Path) -> Result<Self> {
        let mut storage = Self {
            fields_file: fields_file.to_path_buf(),
            fields: BTreeMap::new(),
        };
        storage.load()?;
        Ok(storage)
    }

    pub fn set(&mut self, name: &str, value: &str) -> Result<()> {
        validate_name(name)?;
        self.fields.insert(name.to_string(), value.to_string());
        self.save()
    }

    pub fn get(&self, name: &str) -> Result<Option<String>> {
        validate_name(name)?;
        Ok(self.fields.get(name).cloned())
    }

    pub fn clear(&mut self, name: &str) -> Result<bool> {
        validate_name(name)?;
        let removed = self.fields.remove(name).is_some();
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    pub fn list(&self) -> Vec<(String, String)> {
        self.fields
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }

    pub fn into_map(self) -> BTreeMap<String, String> {
        self.fields
    }

    fn load(&mut self) -> Result<()> {
        if !self.fields_file.exists() {
            return Ok(());
        }

        let raw = fs::read_to_string(&self.fields_file)
            .with_context(|| format!("failed to read fields '{}'", self.fields_file.display()))?;
        self.fields = serde_json::from_str(&raw).context("invalid fields.json")?;
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let payload =
            serde_json::to_vec_pretty(&self.fields).context("failed to serialize fields")?;
        write_atomic(&self.fields_file, &payload)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.fields_file, fs::Permissions::from_mode(0o600))
                .context("failed to set fields file permissions")?;
        }

        Ok(())
    }
}

fn validate_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        return Err(anyhow!("field name cannot be empty"));
    }
    if name != name.trim() {
        return Err(anyhow!(
            "field name cannot have leading or trailing whitespace"
        ));
    }
    Ok(())
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
            "failed to move temporary fields '{}' to '{}'",
            tmp.display(),
            path.display()
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_fields_file(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("forge-fields-test-{name}-{nanos}"))
            .join("fields.json")
    }

    fn cleanup(path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn defaults_when_missing_file() {
        let path = temp_fields_file("defaults");
        let storage = FieldsStorage::new(&path).expect("storage");
        assert!(storage.list().is_empty());
        assert!(!path.exists());
        cleanup(&path);
    }

    #[test]
    fn sets_gets_lists_and_clears_fields() {
        let path = temp_fields_file("roundtrip");
        let mut storage = FieldsStorage::new(&path).expect("storage");

        storage.set("github", "alice").expect("set github");
        storage
            .set("email", "alice@example.com")
            .expect("set email");

        let reloaded = FieldsStorage::new(&path).expect("reload");
        assert_eq!(
            reloaded.get("github").expect("get"),
            Some("alice".to_string())
        );
        assert_eq!(
            reloaded.list(),
            vec![
                ("email".to_string(), "alice@example.com".to_string()),
                ("github".to_string(), "alice".to_string()),
            ]
        );

        let mut reloaded = reloaded;
        assert!(reloaded.clear("github").expect("clear"));
        assert!(!reloaded.clear("github").expect("clear missing"));
        assert_eq!(reloaded.get("github").expect("get"), None);
        cleanup(&path);
    }

    #[test]
    fn rejects_empty_field_names() {
        let path = temp_fields_file("empty-name");
        let mut storage = FieldsStorage::new(&path).expect("storage");
        assert!(storage.set("", "value").is_err());
        assert!(storage.set("  ", "value").is_err());
        cleanup(&path);
    }
}
