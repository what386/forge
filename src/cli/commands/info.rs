use anyhow::Result;

use crate::services::{PathLayout, TemplateResolver, TemplateSource};

pub fn run(template: String) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let resolver = TemplateResolver::new(layout);
    let rec = resolver.resolve(&template, false)?;

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
