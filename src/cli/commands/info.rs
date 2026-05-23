use anyhow::Result;

use crate::cli::commands::scope;
use crate::services::paths::PathLayout;
use crate::templates::{TemplateResolver, TemplateSource};

pub fn run(template: String, local: bool, global: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let source = scope::resolve(&layout, local, global)?;
    let resolver = TemplateResolver::new(layout);
    let rec = resolver.resolve_scoped(&template, source)?;

    println!(
        "{} {}",
        rec.manifest.package.name, rec.manifest.package.version
    );
    println!("{}", rec.manifest.package.description);
    println!();
    if let Some(author) = &rec.manifest.author {
        if let Some(name) = &author.name {
            println!("Author\t{}", name);
        }
    }
    if let Some(repo) = &rec.manifest.package.repository {
        println!("Repository\t{}", repo);
    }
    println!(
        "Source\t{} ({})",
        match rec.source {
            TemplateSource::Local => "local",
            TemplateSource::Global => "global",
        },
        rec.dir.display()
    );
    Ok(())
}
