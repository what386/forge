use anyhow::Result;

use crate::services::paths::PathLayout;
use crate::templates::{TemplateResolver, TemplateSource};

pub fn run(global: bool, local: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let resolver = TemplateResolver::new(layout);

    let (include_local, include_global, include_package) = if local {
        (true, false, false)
    } else if global {
        (false, true, false)
    } else {
        (true, true, true)
    };
    let records = resolver.list(include_local, include_global, include_package)?;

    println!("NAME\tVERSION\tSOURCE\tDESCRIPTION");
    for rec in records {
        let source = match rec.source {
            TemplateSource::Local => "local",
            TemplateSource::Global => "global",
            TemplateSource::Package => "package",
        };
        println!(
            "{}\t{}\t{}\t{}",
            rec.name, rec.manifest.package.version, source, rec.manifest.package.description
        );
    }
    Ok(())
}
