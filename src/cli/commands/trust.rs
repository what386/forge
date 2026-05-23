use anyhow::Result;

use crate::storage::paths::PathLayout;
use crate::storage::trust::TrustManager;
use crate::templates::TemplateResolver;

pub fn run_add(template: String, global: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let resolver = TemplateResolver::new(layout.clone());
    let rec = resolver.resolve(&template, global)?;

    let tm = TrustManager::new(layout.trust_file);
    tm.trust_dir(&rec.dir)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    println!("trusted {}", template);
    Ok(())
}

pub fn run_remove(template: String) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let resolver = TemplateResolver::new(layout.clone());
    let rec = resolver.resolve(&template, false)?;

    let tm = TrustManager::new(layout.trust_file);
    let removed = tm
        .revoke_dir(&rec.dir)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    if removed {
        println!("removed trust for {}", template);
    } else {
        println!("no trust entry for {}", template);
    }
    Ok(())
}

pub fn run_list() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let layout = PathLayout::discover(cwd)?;
    let tm = TrustManager::new(layout.trust_file);
    let entries = tm
        .list_entries()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    if entries.is_empty() {
        println!("no trusted templates");
        return Ok(());
    }
    for entry in entries {
        let trusted = tm
            .is_dir_trusted(std::path::Path::new(&entry.path))
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        if trusted {
            println!("{}\t{}", entry.path, entry.checksum);
        } else {
            println!("{}\tINVALIDATED", entry.path);
        }
    }
    Ok(())
}
