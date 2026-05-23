use anyhow::{bail, Context, Result};
use std::collections::BTreeSet;
use std::io::Write;

use crate::packages::{fetch_repo, probe_repo, PackageManager};

pub fn run_probe(repo: String) -> Result<()> {
    let fetched = fetch_repo(&repo)?;
    let probed = probe_repo(fetched.path())?;
    let repo_name = repo.rsplit('/').next().unwrap_or(repo.as_str());
    println!("{} ({})", repo_name.trim_end_matches(".git"), repo);
    println!();
    for template in probed.templates {
        println!(
            "  {}\t{}\t{}",
            template.name, template.version, template.description
        );
    }
    Ok(())
}

pub fn run_install(repo: String, mut templates: Vec<String>, interactive: bool) -> Result<()> {
    let fetched = fetch_repo(&repo)?;
    let probed = probe_repo(fetched.path())?;
    if templates.is_empty() {
        if !interactive {
            bail!("provide at least one template or use --interactive");
        }
        templates = select_interactively(&probed.templates)?;
    }

    let manager = PackageManager::global()?;
    let installed = manager.install_templates(&repo, &probed, &templates)?;
    for entry in installed {
        println!(
            "installed {}\t{}\t{}",
            entry.name,
            repo,
            entry.destination.display()
        );
    }
    Ok(())
}

pub fn run_remove(names: Vec<String>) -> Result<()> {
    let manager = PackageManager::global()?;
    let removed = if names.is_empty() {
        manager.remove_all()?
    } else {
        manager.remove_templates(&names)?
    };
    if removed.is_empty() {
        println!("no packages removed");
        return Ok(());
    }
    for name in removed {
        println!("removed {}", name);
    }
    Ok(())
}

pub fn run_update(names: Vec<String>) -> Result<()> {
    let manager = PackageManager::global()?;
    let entries = manager.list()?;
    let wanted: Option<BTreeSet<String>> = if names.is_empty() {
        None
    } else {
        Some(names.into_iter().collect())
    };
    let mut updated = Vec::new();
    for (name, record) in entries {
        if let Some(w) = &wanted {
            if !w.contains(&name) {
                continue;
            }
        }
        let fetched = fetch_repo(&record.repo)
            .with_context(|| format!("failed to fetch repo for '{}'", name))?;
        let probed = probe_repo(fetched.path())
            .with_context(|| format!("failed to probe repo for '{}'", name))?;
        manager
            .install_templates(&record.repo, &probed, std::slice::from_ref(&name))
            .with_context(|| format!("failed to update '{}'", name))?;
        updated.push(name);
    }
    if updated.is_empty() {
        println!("no packages updated");
        return Ok(());
    }
    updated.sort();
    for name in updated {
        println!("updated {}", name);
    }
    Ok(())
}

pub fn run_list() -> Result<()> {
    let manager = PackageManager::global()?;
    let entries = manager.list()?;
    if entries.is_empty() {
        println!("no installed packages");
        return Ok(());
    }
    println!("NAME\tREPO\tREF\tINSTALLED_AT");
    for (name, rec) in entries {
        println!(
            "{}\t{}\t{}\t{}",
            name, rec.repo, rec.ref_name, rec.installed_at
        );
    }
    Ok(())
}

fn select_interactively(templates: &[crate::packages::ProbedTemplate]) -> Result<Vec<String>> {
    if templates.is_empty() {
        bail!("repository contains no templates");
    }
    println!("select templates to install (comma-separated names):");
    for t in templates {
        println!("  {} ({})", t.name, t.version);
    }
    print!("templates: ");
    std::io::stdout()
        .flush()
        .context("failed to flush stdout")?;
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("failed to read selection")?;
    let selected: Vec<String> = line
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    if selected.is_empty() {
        bail!("no templates selected");
    }
    Ok(selected)
}
