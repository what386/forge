use anyhow::{anyhow, Context, Result};
use std::process::Command;
use toml::Value;

use crate::storage::config::ConfigStorage;
use crate::storage::paths::PathLayout;

pub fn run_set(key: String, value: String) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let mut storage = ConfigStorage::new(&layout.config_file)?;
    storage.try_set_value(&key, &value)?;
    println!("set {}", key);
    Ok(())
}

pub fn run_get(key: String) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let storage = ConfigStorage::new(&layout.config_file)?;
    let value = storage.try_get_value(&key)?;
    println!("{}", value_to_output(&value));
    Ok(())
}

pub fn run_list() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let storage = ConfigStorage::new(&layout.config_file)?;
    let mut rows: Vec<(String, String)> = storage.get_flattened_config().into_iter().collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    for (key, value) in rows {
        println!("{}={}", key, value);
    }
    Ok(())
}

pub fn run_edit() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let storage = ConfigStorage::new(&layout.config_file)?;
    if !storage.config_path().exists() {
        storage.save_config()?;
    }

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = Command::new(&editor)
        .arg(storage.config_path())
        .status()
        .with_context(|| format!("failed to launch editor '{}'", editor))?;
    if status.success() {
        return Ok(());
    }

    Err(anyhow!("editor exited with status: {}", status))
}

fn value_to_output(value: &Value) -> String {
    match value {
        Value::String(v) => v.clone(),
        other => other.to_string(),
    }
}
