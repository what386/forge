use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "forge")]
#[command(about = "A Lua-powered template engine.")]
#[command(long_about = "Forge scaffolds new projects from Lua templates.\n\n\
    Templates live in .forge/templates/ (local) or ~/.forge/templates/ (global).\n\n\
    EXAMPLES:\n  \
    forge new webapp my-app\n  \
    forge list\n  \
    forge info webapp\n  \
    forge create my-template\n  \
    forge check webapp")]
#[command(
    version,
    long_version = concat!(env!("CARGO_PKG_VERSION"), " (", env!("GIT_HASH"), ")")
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scaffold a new project from a template
    #[command(long_about = "Scaffold a new project from a template.\n\n\
        Locates the template, validates its manifest, checks required commands,\n\
        prompts for any elevated permissions, then executes main.lua.\n\n\
        EXAMPLES:\n  \
        forge new webapp my-app\n  \
        forge new fullstack my-project")]
    New {
        /// Name of the template to use
        template: String,
        /// Name of the project to create
        name: String,
        /// Use default values for all prompts
        #[arg(long)]
        default: bool,
    },

    /// List available templates
    #[command(long_about = "List all available templates.\n\n\
        Searches both .forge/templates/ (local) and ~/.forge/templates/ (global).\n\
        Local templates are listed first and take precedence over global ones.\n\n\
        EXAMPLES:\n  \
        forge list\n  \
        forge list --local\n  \
        forge list --global")]
    List {
        /// Show only global templates
        #[arg(short = 'g', long, conflicts_with = "local")]
        global: bool,
        /// Show only local templates
        #[arg(short = 'l', long, conflicts_with = "global")]
        local: bool,
    },

    /// Print details about a template
    #[command(long_about = "Print details about a template.\n\n\
        Reads and displays the template's manifest.toml in a human-readable format,\n\
        including version, description, author, tags, and required permissions.\n\n\
        EXAMPLES:\n  \
        forge info webapp\n  \
        forge info fullstack")]
    Info {
        /// Name of the template to inspect
        template: String,
    },

    /// Scaffold a new blank template
    #[command(long_about = "Scaffold a new blank template.\n\n\
        Generates a minimal template in .forge/templates/<name>/ (or globally with --global)\n\
        with a pre-populated manifest.toml and a commented main.lua to get started.\n\n\
        EXAMPLES:\n  \
        forge create my-template\n  \
        forge create my-template --global")]
    Create {
        /// Name of the template to create
        name: String,
        /// Create in ~/.forge/templates/ instead of .forge/templates/
        #[arg(short = 'g', long)]
        global: bool,
    },

    /// Remove local template(s)
    #[command(long_about = "Remove one or more local templates.\n\n\
        Deletes template directories from .forge/templates/ and removes them\n\
        from .forge/templates.json.\n\n\
        EXAMPLES:\n  \
        forge remove webapp\n  \
        forge remove webapp fullstack")]
    Remove {
        /// Local template name(s) to remove
        #[arg(required = true)]
        names: Vec<String>,
    },

    /// Check a template without executing it
    #[command(long_about = "Check a template without executing it.\n\n\
        Checks that manifest.toml is present and valid, main.lua parses without\n\
        syntax errors, all files referenced by render calls exist, and all\n\
        declared permissions are known values.\n\n\
        EXAMPLES:\n  \
        forge check webapp\n  \
        forge check my-template --global")]
    Check {
        /// Name of the template to check
        template: String,
        /// Check a template in ~/.forge/templates/
        #[arg(short = 'g', long)]
        global: bool,
    },

    /// Manage template trust
    #[command(long_about = "Manage which templates are trusted.\n\n\
        Trusted templates skip the permission confirmation prompt. Trust is tied\n\
        to a checksum of the entire template directory — if any file changes,\n\
        the trust entry is invalidated and the prompt reappears.\n\n\
        EXAMPLES:\n  \
        forge trust add webapp\n  \
        forge trust remove webapp\n  \
        forge trust list")]
    Trust {
        #[command(subcommand)]
        action: TrustAction,
    },

    /// Manage Forge configuration
    #[command(
        long_about = "Manage Forge configuration stored in ~/.forge/config.toml.\n\n\
        EXAMPLES:\n  \
        forge config set user.name \"Alice\"\n  \
        forge config set user.email \"alice@example.com\"\n  \
        forge config get user.name\n  \
        forge config list\n  \
        forge config edit"
    )]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Manage remote template packages
    #[command(long_about = "Manage remote template packages.\n\n\
        EXAMPLES:\n  \
        forge package probe https://github.com/alice/forge-templates.git\n  \
        forge package install https://github.com/alice/forge-templates.git fullstack\n  \
        forge package remove fullstack\n  \
        forge package update\n  \
        forge package list")]
    Package {
        #[command(subcommand)]
        action: PackageAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum TrustAction {
    /// Trust a template and store its checksum
    #[command(long_about = "Mark a template as trusted.\n\n\
        Computes a checksum of the entire template directory and stores it in\n\
        ~/.forge/trust.json. Trusted templates skip the permission prompt.\n\
        If the template changes, the trust entry is invalidated automatically.\n\n\
        EXAMPLES:\n  \
        forge trust add webapp\n  \
        forge trust add fullstack --global")]
    Add {
        /// Name of the template to trust
        template: String,
        /// Trust a template in ~/.forge/templates/
        #[arg(short = 'g', long)]
        global: bool,
    },

    /// Revoke trust from a template
    #[command(long_about = "Remove a template's trust entry.\n\n\
        The template can still be run, but permission prompts will reappear.\n\n\
        EXAMPLES:\n  \
        forge trust remove webapp")]
    Remove {
        /// Name of the template to untrust
        template: String,
    },

    /// List all trusted templates
    #[command(
        long_about = "Show all trusted templates and their stored checksums.\n\n\
        Templates whose checksum no longer matches their current state are shown\n\
        as invalidated.\n\n\
        EXAMPLES:\n  \
        forge trust list"
    )]
    List,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Set a config key
    Set {
        /// Dot-separated config key path
        key: String,
        /// Config value (TOML literal or plain string)
        value: String,
    },
    /// Get a config key
    Get {
        /// Dot-separated config key path
        key: String,
    },
    /// List all config keys
    List,
    /// Open config.toml in $EDITOR
    Edit,
}

#[derive(Subcommand, Debug)]
pub enum PackageAction {
    /// Probe templates from a remote repository
    Probe {
        /// Git URL for the template repository
        repo: String,
    },
    /// Install templates from a remote repository
    Install {
        /// Git URL for the template repository
        repo: String,
        /// Template name(s) to install
        templates: Vec<String>,
        /// Prompt to select template(s) when names are omitted
        #[arg(long)]
        interactive: bool,
    },
    /// Remove installed template package(s)
    Remove {
        /// Installed package/template names to remove (all if omitted)
        names: Vec<String>,
    },
    /// Update installed template package(s)
    Update {
        /// Installed package/template names to update (all if omitted)
        names: Vec<String>,
    },
    /// List installed template packages
    List,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn new_parses_template_and_name() {
        let cli = Cli::parse_from(["forge", "new", "webapp", "my-app"]);
        match cli.command {
            Commands::New {
                template,
                name,
                default,
            } => {
                assert_eq!(template, "webapp");
                assert_eq!(name, "my-app");
                assert!(!default);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn new_parses_default_flag() {
        let cli = Cli::parse_from(["forge", "new", "webapp", "my-app", "--default"]);
        match cli.command {
            Commands::New {
                template,
                name,
                default,
            } => {
                assert_eq!(template, "webapp");
                assert_eq!(name, "my-app");
                assert!(default);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn list_parses_global_flag() {
        let cli = Cli::parse_from(["forge", "list", "--global"]);
        match cli.command {
            Commands::List { global, local } => {
                assert!(global);
                assert!(!local);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn list_parses_local_flag() {
        let cli = Cli::parse_from(["forge", "list", "--local"]);
        match cli.command {
            Commands::List { global, local } => {
                assert!(!global);
                assert!(local);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn list_rejects_global_and_local_together() {
        assert!(Cli::try_parse_from(["forge", "list", "--global", "--local"]).is_err());
    }

    #[test]
    fn list_parses_global_short_flag() {
        let cli = Cli::parse_from(["forge", "list", "-g"]);
        match cli.command {
            Commands::List { global, local } => {
                assert!(global);
                assert!(!local);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn list_parses_local_short_flag() {
        let cli = Cli::parse_from(["forge", "list", "-l"]);
        match cli.command {
            Commands::List { global, local } => {
                assert!(!global);
                assert!(local);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn list_rejects_global_and_local_short_together() {
        assert!(Cli::try_parse_from(["forge", "list", "-g", "-l"]).is_err());
    }

    #[test]
    fn info_parses_template_name() {
        let cli = Cli::parse_from(["forge", "info", "fullstack"]);
        match cli.command {
            Commands::Info { template } => assert_eq!(template, "fullstack"),
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn create_parses_name_and_global_flag() {
        let cli = Cli::parse_from(["forge", "create", "my-template", "--global"]);
        match cli.command {
            Commands::Create { name, global } => {
                assert_eq!(name, "my-template");
                assert!(global);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn create_parses_name_and_global_short_flag() {
        let cli = Cli::parse_from(["forge", "create", "my-template", "-g"]);
        match cli.command {
            Commands::Create { name, global } => {
                assert_eq!(name, "my-template");
                assert!(global);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn check_parses_template_and_global_flag() {
        let cli = Cli::parse_from(["forge", "check", "webapp", "--global"]);
        match cli.command {
            Commands::Check { template, global } => {
                assert_eq!(template, "webapp");
                assert!(global);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn remove_parses_one_or_more_names() {
        let one = Cli::parse_from(["forge", "remove", "webapp"]);
        assert!(matches!(
            one.command,
            Commands::Remove { names } if names == vec!["webapp".to_string()]
        ));

        let many = Cli::parse_from(["forge", "remove", "webapp", "fullstack"]);
        assert!(matches!(
            many.command,
            Commands::Remove { names }
                if names == vec!["webapp".to_string(), "fullstack".to_string()]
        ));
    }

    #[test]
    fn check_parses_template_and_global_short_flag() {
        let cli = Cli::parse_from(["forge", "check", "webapp", "-g"]);
        match cli.command {
            Commands::Check { template, global } => {
                assert_eq!(template, "webapp");
                assert!(global);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn trust_add_parses_template_and_global_flag() {
        let cli = Cli::parse_from(["forge", "trust", "add", "webapp", "--global"]);
        match cli.command {
            Commands::Trust {
                action: TrustAction::Add { template, global },
            } => {
                assert_eq!(template, "webapp");
                assert!(global);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn trust_add_parses_template_and_global_short_flag() {
        let cli = Cli::parse_from(["forge", "trust", "add", "webapp", "-g"]);
        match cli.command {
            Commands::Trust {
                action: TrustAction::Add { template, global },
            } => {
                assert_eq!(template, "webapp");
                assert!(global);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn trust_remove_parses_template() {
        let cli = Cli::parse_from(["forge", "trust", "remove", "webapp"]);
        match cli.command {
            Commands::Trust {
                action: TrustAction::Remove { template },
            } => assert_eq!(template, "webapp"),
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn trust_list_parses() {
        let cli = Cli::parse_from(["forge", "trust", "list"]);
        assert!(matches!(
            cli.command,
            Commands::Trust {
                action: TrustAction::List
            }
        ));
    }

    #[test]
    fn config_set_parses_key_and_value() {
        let cli = Cli::parse_from(["forge", "config", "set", "user.name", "Alice"]);
        assert!(matches!(
            cli.command,
            Commands::Config {
                action: ConfigAction::Set { key, value }
            } if key == "user.name" && value == "Alice"
        ));
    }

    #[test]
    fn config_get_parses_key() {
        let cli = Cli::parse_from(["forge", "config", "get", "user.email"]);
        assert!(matches!(
            cli.command,
            Commands::Config {
                action: ConfigAction::Get { key }
            } if key == "user.email"
        ));
    }

    #[test]
    fn config_list_parses() {
        let cli = Cli::parse_from(["forge", "config", "list"]);
        assert!(matches!(
            cli.command,
            Commands::Config {
                action: ConfigAction::List
            }
        ));
    }

    #[test]
    fn config_edit_parses() {
        let cli = Cli::parse_from(["forge", "config", "edit"]);
        assert!(matches!(
            cli.command,
            Commands::Config {
                action: ConfigAction::Edit
            }
        ));
    }

    #[test]
    fn package_probe_parses_repo() {
        let cli = Cli::parse_from([
            "forge",
            "package",
            "probe",
            "https://github.com/alice/forge-templates.git",
        ]);
        assert!(matches!(
            cli.command,
            Commands::Package {
                action: PackageAction::Probe { repo }
            } if repo == "https://github.com/alice/forge-templates.git"
        ));
    }

    #[test]
    fn package_install_parses_repo_templates_and_interactive() {
        let cli = Cli::parse_from([
            "forge",
            "package",
            "install",
            "https://github.com/alice/forge-templates.git",
            "fullstack",
            "rust",
            "--interactive",
        ]);
        assert!(matches!(
            cli.command,
            Commands::Package {
                action: PackageAction::Install {
                    repo,
                    templates,
                    interactive,
                }
            } if repo == "https://github.com/alice/forge-templates.git"
                && templates == vec!["fullstack".to_string(), "rust".to_string()]
                && interactive
        ));
    }

    #[test]
    fn package_remove_and_update_parse_optional_names() {
        let remove_cli = Cli::parse_from(["forge", "package", "remove", "fullstack", "rust"]);
        assert!(matches!(
            remove_cli.command,
            Commands::Package {
                action: PackageAction::Remove { names }
            } if names == vec!["fullstack".to_string(), "rust".to_string()]
        ));

        let update_cli = Cli::parse_from(["forge", "package", "update"]);
        assert!(matches!(
            update_cli.command,
            Commands::Package {
                action: PackageAction::Update { names }
            } if names.is_empty()
        ));
    }
}
