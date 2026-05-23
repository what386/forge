use anyhow::Result;

use crate::cli::arguments::{Cli, Commands, TrustAction};
use crate::cli::commands;

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::New { template, name } => commands::new::run(template, name),
            Commands::List { global, local } => commands::list::run(global, local),
            Commands::Info { template } => commands::info::run(template),
            Commands::Create { name, global } => commands::create::run(name, global),
            Commands::Check { template, global } => commands::validate::run(template, global),
            Commands::Trust { action } => match action {
                TrustAction::Add { template, global } => commands::trust::run_add(template, global),
                TrustAction::Remove { template } => commands::trust::run_remove(template),
                TrustAction::List => commands::trust::run_list(),
            },
        }
    }
}
