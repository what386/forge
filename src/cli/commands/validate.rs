use anyhow::Result;

use crate::services::paths::PathLayout;
use crate::templates::{validate_template, TemplateResolver};

pub fn run(template: String, global: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let resolver = TemplateResolver::new(layout);
    let rec = resolver.resolve(&template, global)?;

    let errors = validate_template(&rec)?;
    if errors.is_empty() {
        println!("✓ manifest.toml valid");
        println!("✓ main.lua parses");
        println!("✓ all referenced files exist");
        println!("  {} is valid", template);
        return Ok(());
    }

    for err in &errors {
        println!("✗ {}", err);
    }
    anyhow::bail!("{} has {} errors", template, errors.len());
}
