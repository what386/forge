use anyhow::Result;

use crate::services::{run_template, PathLayout, TemplateResolver};

pub fn run(template: String, name: String) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd.clone())?;
    let resolver = TemplateResolver::new(layout);
    let rec = resolver.resolve(&template, false)?;
    run_template(&rec, &name, &cwd)
}
