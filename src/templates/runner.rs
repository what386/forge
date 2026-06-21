use anyhow::{anyhow, bail, Result};
use std::path::Path;

use crate::lua::{Logger, Runtime, RuntimeConfig};
use crate::services::paths::PathLayout;
use crate::services::prompts::{DefaultPrompts, StdioPrompts};
use crate::services::storage::fields::FieldsStorage;
use crate::services::storage::trust::TrustManager;
use crate::templates::manifest::Permission;
use crate::templates::resolver::TemplateRecord;

pub fn run_template(
    record: &TemplateRecord,
    project_name: &str,
    cwd: &Path,
    use_defaults: bool,
) -> Result<()> {
    let project_dir = cwd.join(project_name);
    if project_dir.exists() {
        bail!("output directory already exists: {}", project_dir.display());
    }

    let main_lua = record.dir.join("main.lua");
    if !main_lua.is_file() {
        bail!("template is missing main.lua");
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
    let allowed_programs = record
        .manifest
        .requires
        .as_ref()
        .map(|r| r.programs.clone())
        .unwrap_or_default();

    let layout = PathLayout::discover(cwd.to_path_buf())?;

    if !permissions.is_empty() || !allowed_programs.is_empty() {
        let trust = TrustManager::new(layout.trust_file.clone());
        let trusted = trust
            .is_dir_trusted(&record.dir)
            .map_err(|e| anyhow!(e.to_string()))?;
        if !trusted {
            print_permission_summary(
                &record.name,
                &permissions,
                &allowed_commands,
                &allowed_programs,
            );
            let choice = if use_defaults {
                TrustChoice::RunOnce
            } else {
                confirm_trust_stdin("Trust this template? (y = trust, n = run once, q = quit): ")?
            };
            match choice {
                TrustChoice::Trust => trust
                    .trust_dir(&record.dir)
                    .map_err(|e| anyhow!(format!("failed to persist trust: {}", e)))?,
                TrustChoice::RunOnce => {
                    if use_defaults {
                        eprintln!("using --default: continuing with run-once (not trusted)");
                    }
                }
                TrustChoice::Quit => bail!("aborted by user"),
            }
        }
    }

    std::fs::create_dir_all(&project_dir)
        .map_err(|e| anyhow!("failed to create {}: {}", project_dir.display(), e))?;

    let mut runtime = Runtime::new(RuntimeConfig {
        project_name: project_name.to_string(),
        project_dir,
        template_name: record.name.clone(),
        template_dir: record.dir.clone(),
        allowed_commands,
        allowed_programs,
        fields: FieldsStorage::new(&layout.fields_file)?.into_map(),
        permissions,
        logger: Some(std::sync::Arc::new(StdioLogger {})),
        prompts: if use_defaults {
            Some(std::sync::Arc::new(DefaultPrompts {}))
        } else {
            Some(std::sync::Arc::new(StdioPrompts {}))
        },
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

fn print_permission_summary(
    template_name: &str,
    permissions: &[Permission],
    allowed_commands: &[String],
    allowed_programs: &[String],
) {
    eprintln!(
        "Template \"{}\" is requesting elevated permissions:\n",
        template_name
    );

    if permissions.contains(&Permission::Execution) {
        if allowed_commands.is_empty() {
            eprintln!("  • exec        — may execute external commands");
        } else {
            eprintln!(
                "  • exec        — may execute external commands: {}",
                allowed_commands.join(", ")
            );
        }
    }
    if permissions.contains(&Permission::EscapeCwd) {
        eprintln!("  • escape_cwd  — may read and write paths outside the project directory (!!!)");
    }
    if permissions.contains(&Permission::Network) {
        eprintln!("  • network     — may make network requests");
    }
    if permissions.contains(&Permission::ReadEnv) {
        eprintln!(
            "  • read_env    — may read environment variables beyond: HOME, USER, PATH, SHELL"
        );
    }
    if !allowed_programs.is_empty() {
        eprintln!(
            "  • program     — use the following program APIs: {}",
            allowed_programs.join(", ")
        );
    }
}

enum TrustChoice {
    Trust,
    RunOnce,
    Quit,
}

fn confirm_trust_stdin(prompt: &str) -> Result<TrustChoice> {
    use std::io::Write;
    loop {
        print!("{}", prompt);
        std::io::stdout().flush()?;
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        let ans = line.trim().to_ascii_lowercase();
        match ans.as_str() {
            "y" | "yes" => return Ok(TrustChoice::Trust),
            "n" | "no" => return Ok(TrustChoice::RunOnce),
            "q" | "quit" => return Ok(TrustChoice::Quit),
            _ => {
                eprintln!("please respond with y, n, or q");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::manifest::{Manifest, Package};
    use crate::templates::resolver::TemplateSource;

    #[test]
    fn missing_main_lua_does_not_create_project_dir() {
        let tmp = tempfile::tempdir().expect("tmp");
        let template_dir = tmp.path().join("broken");
        std::fs::create_dir_all(&template_dir).expect("create template dir");
        let record = TemplateRecord {
            name: "broken".to_string(),
            dir: template_dir,
            source: TemplateSource::Local,
            manifest: Manifest {
                package: Package {
                    name: "broken".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Broken template".to_string(),
                    min_forge_version: "0.1.0".to_string(),
                    repository: None,
                },
                author: None,
                tags: None,
                requires: None,
            },
        };

        let err = run_template(&record, "app", tmp.path(), true).expect_err("missing main.lua");

        assert!(format!("{err:#}").contains("template is missing main.lua"));
        assert!(!tmp.path().join("app").exists());
    }
}
