use anyhow::Result;

use crate::services::paths::PathLayout;
use crate::templates::{TemplateResolver, TemplateSource};

pub fn run(global: bool, local: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let resolver = TemplateResolver::new(layout);

    let include_local = !global;
    let include_global = !local;
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
