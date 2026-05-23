use anyhow::Result;

use crate::cli::commands::scope;
use crate::services::paths::PathLayout;
use crate::templates::{run_template, TemplateResolver};

pub fn run(
    template: String,
    name: String,
    local: bool,
    global: bool,
    use_defaults: bool,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd.clone())?;
    let source = scope::resolve(&layout, local, global)?;
    let resolver = TemplateResolver::new(layout);
    let rec = resolver.resolve_scoped(&template, source)?;
    run_template(&rec, &name, &cwd, use_defaults)
}
