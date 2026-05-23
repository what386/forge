use anyhow::Result;

use crate::cli::arguments::{Cli, Commands, ConfigAction, PackageAction, TrustAction};
use crate::cli::commands;

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::New {
                template,
                name,
                default,
            } => commands::new::run(template, name, default),
            Commands::List { global, local } => commands::list::run(global, local),
            Commands::Info { template } => commands::info::run(template),
            Commands::Create { name, global } => commands::create::run(name, global),
            Commands::Remove { names } => commands::remove::run(names),
            Commands::Check { template, global } => commands::validate::run(template, global),
            Commands::Trust { action } => match action {
                TrustAction::Add { template, global } => commands::trust::run_add(template, global),
                TrustAction::Remove { template } => commands::trust::run_remove(template),
                TrustAction::List => commands::trust::run_list(),
            },
            Commands::Config { action } => match action {
                ConfigAction::Set { key, value } => commands::config::run_set(key, value),
                ConfigAction::Get { key } => commands::config::run_get(key),
                ConfigAction::List => commands::config::run_list(),
                ConfigAction::Edit => commands::config::run_edit(),
            },
            Commands::Package { action } => match action {
                PackageAction::Probe { repo } => commands::package::run_probe(repo),
                PackageAction::Install {
                    repo,
                    templates,
                    interactive,
                } => commands::package::run_install(repo, templates, interactive),
                PackageAction::Remove { names } => commands::package::run_remove(names),
                PackageAction::Update { names } => commands::package::run_update(names),
                PackageAction::List => commands::package::run_list(),
            },
        }
    }
}
