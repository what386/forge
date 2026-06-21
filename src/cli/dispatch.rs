use anyhow::Result;

use crate::cli::arguments::{
    Cli, Commands, ConfigAction, FieldsAction, PackageAction, TrustAction,
};
use crate::cli::commands;

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::New {
                template,
                name,
                local,
                global,
                default,
            } => commands::new::run(template, name, local, global, default),
            Commands::List { global, local } => commands::list::run(global, local),
            Commands::Info {
                template,
                local,
                global,
            } => commands::info::run(template, local, global),
            Commands::Create {
                name,
                local,
                global,
            } => commands::create::run(name, local, global),
            Commands::Remove {
                names,
                local,
                global,
            } => commands::remove::run(names, local, global),
            Commands::Check {
                template,
                local,
                global,
            } => commands::validate::run(template, local, global),
            Commands::Trust { action } => match action {
                TrustAction::Add {
                    template,
                    local,
                    global,
                } => commands::trust::run_add(template, local, global),
                TrustAction::Remove {
                    template,
                    local,
                    global,
                } => commands::trust::run_remove(template, local, global),
                TrustAction::List => commands::trust::run_list(),
            },
            Commands::Config { action } => match action {
                ConfigAction::Set { key, value } => commands::config::run_set(key, value),
                ConfigAction::Get { key } => commands::config::run_get(key),
                ConfigAction::List => commands::config::run_list(),
                ConfigAction::Edit => commands::config::run_edit(),
            },
            Commands::Fields { action } => match action {
                FieldsAction::Set { assignment } => commands::fields::run_set(assignment),
                FieldsAction::Get { name } => commands::fields::run_get(name),
                FieldsAction::Clear { name } => commands::fields::run_clear(name),
                FieldsAction::List => commands::fields::run_list(),
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
