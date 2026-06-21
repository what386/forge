use anyhow::{anyhow, Result};

use crate::services::paths::PathLayout;
use crate::services::storage::fields::FieldsStorage;

pub fn run_set(assignment: String) -> Result<()> {
    let (name, value) = parse_assignment(&assignment)?;
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let mut storage = FieldsStorage::new(&layout.fields_file)?;
    storage.set(name, value)?;
    println!("set {}", name);
    Ok(())
}

pub fn run_get(name: String) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let storage = FieldsStorage::new(&layout.fields_file)?;
    let value = storage
        .get(&name)?
        .ok_or_else(|| anyhow!("field not found: {}", name))?;
    println!("{}", value);
    Ok(())
}

pub fn run_clear(name: String) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let mut storage = FieldsStorage::new(&layout.fields_file)?;
    if storage.clear(&name)? {
        println!("cleared {}", name);
    } else {
        println!("no field named {}", name);
    }
    Ok(())
}

pub fn run_list() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let storage = FieldsStorage::new(&layout.fields_file)?;
    for (name, value) in storage.list() {
        println!("{}={}", name, value);
    }
    Ok(())
}

fn parse_assignment(assignment: &str) -> Result<(&str, &str)> {
    let (name, value) = assignment
        .split_once('=')
        .ok_or_else(|| anyhow!("field assignment must be NAME=VALUE"))?;
    if name.trim().is_empty() {
        return Err(anyhow!("field name cannot be empty"));
    }
    if name != name.trim() {
        return Err(anyhow!(
            "field name cannot have leading or trailing whitespace"
        ));
    }
    Ok((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_assignment_with_equals_in_value() {
        assert_eq!(
            parse_assignment("github.token=foo=bar").expect("assignment"),
            ("github.token", "foo=bar")
        );
    }

    #[test]
    fn rejects_assignment_without_equals() {
        assert!(parse_assignment("github.username").is_err());
    }
}
