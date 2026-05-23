use anyhow::{anyhow, bail, Result};
use std::path::Path;

use crate::lua::{Logger, Runtime, RuntimeConfig};
use crate::services::prompts::StdioPrompts;
use crate::storage::paths::PathLayout;
use crate::storage::trust::TrustManager;
use crate::templates::resolver::TemplateRecord;

pub fn run_template(record: &TemplateRecord, project_name: &str, cwd: &Path) -> Result<()> {
    let project_dir = cwd.join(project_name);
    if project_dir.exists() {
        bail!("output directory already exists: {}", project_dir.display());
    }

    let permissions = record
        .manifest
        .requires
        .as_ref()
        .map(|r| r.permissions.clone())
        .unwrap_or_default();
    let allowed_commands = record
        .manifest
        .requires
        .as_ref()
        .map(|r| r.commands.clone())
        .unwrap_or_default();

    if !permissions.is_empty() {
        let trust = TrustManager::new(PathLayout::discover(cwd.to_path_buf())?.trust_file);
        let trusted = trust.is_dir_trusted(&record.dir).unwrap_or(false);
        if !trusted {
            eprintln!(
                "Template '{}' requests elevated permissions: {:?}",
                record.name, permissions
            );
            if !confirm_stdin("Proceed? (y/n): ")? {
                bail!("aborted by user");
            }
        }
    }

    std::fs::create_dir_all(&project_dir)
        .map_err(|e| anyhow!("failed to create {}: {}", project_dir.display(), e))?;

    let main_lua = record.dir.join("main.lua");
    if !main_lua.is_file() {
        bail!("template is missing main.lua");
    }

    let mut runtime = Runtime::new(RuntimeConfig {
        project_name: project_name.to_string(),
        project_dir,
        template_name: record.name.clone(),
        template_dir: record.dir.clone(),
        allowed_commands,
        permissions,
        logger: Some(std::sync::Arc::new(StdioLogger {})),
        prompts: Some(std::sync::Arc::new(StdioPrompts {})),
        ..RuntimeConfig::default()
    });

    runtime
        .run(&main_lua.to_string_lossy())
        .map_err(|e| anyhow!(e.to_string()))
}

struct StdioLogger;

impl Logger for StdioLogger {
    fn info(&self, msg: &str) {
        println!("{}", msg);
    }

    fn warn(&self, msg: &str) {
        eprintln!("warn: {}", msg);
    }

    fn error(&self, msg: &str) {
        eprintln!("error: {}", msg);
    }

    fn success(&self, msg: &str) {
        println!("{}", msg);
    }
}

fn confirm_stdin(prompt: &str) -> Result<bool> {
    use std::io::Write;
    print!("{}", prompt);
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let ans = line.trim().to_ascii_lowercase();
    Ok(ans == "y" || ans == "yes")
}
