use anyhow::Result;

use crate::cli::commands::scope;
use crate::services::paths::PathLayout;
use crate::templates::{TemplateResolver, TemplateSource};

pub fn run(global: bool, local: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let default_scope = scope::resolve(&layout, false, false)?;
    let resolver = TemplateResolver::new(layout);

    let (include_local, include_global) = if local {
        (true, false)
    } else if global {
        (false, true)
    } else {
        match default_scope {
            crate::templates::ResolveScope::Local => (true, false),
            crate::templates::ResolveScope::Global => (false, true),
        }
    };
    let records = resolver.list(include_local, include_global)?;

    println!("NAME\tVERSION\tSOURCE\tDESCRIPTION");
    for rec in records {
        let source = match rec.source {
            TemplateSource::Local => "local",
            TemplateSource::Global => "global",
        };
        println!(
            "{}\t{}\t{}\t{}",
            rec.name, rec.manifest.package.version, source, rec.manifest.package.description
        );
    }
    Ok(())
}
