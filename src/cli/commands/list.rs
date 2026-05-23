use anyhow::Result;

use crate::services::{PathLayout, TemplateResolver, TemplateSource};

pub fn run(global: bool, local: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let resolver = TemplateResolver::new(layout);

    let include_local = if global { false } else { true };
    let include_global = if local { false } else { true };
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
