# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_forge_global_optspecs
	string join \n h/help V/version
end

function __fish_forge_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_forge_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_forge_using_subcommand
	set -l cmd (__fish_forge_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c forge -n "__fish_forge_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c forge -n "__fish_forge_needs_command" -s V -l version -d 'Print version'
complete -c forge -n "__fish_forge_needs_command" -f -a "new" -d 'Scaffold a new project from a template'
complete -c forge -n "__fish_forge_needs_command" -f -a "list" -d 'List available templates'
complete -c forge -n "__fish_forge_needs_command" -f -a "info" -d 'Print details about a template'
complete -c forge -n "__fish_forge_needs_command" -f -a "create" -d 'Scaffold a new blank template'
complete -c forge -n "__fish_forge_needs_command" -f -a "remove" -d 'Remove template(s)'
complete -c forge -n "__fish_forge_needs_command" -f -a "check" -d 'Check a template without executing it'
complete -c forge -n "__fish_forge_needs_command" -f -a "trust" -d 'Manage template trust'
complete -c forge -n "__fish_forge_needs_command" -f -a "config" -d 'Manage Forge configuration'
complete -c forge -n "__fish_forge_needs_command" -f -a "package" -d 'Manage remote template packages'
complete -c forge -n "__fish_forge_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c forge -n "__fish_forge_using_subcommand new" -s l -l local -d 'Use local .forge/templates'
complete -c forge -n "__fish_forge_using_subcommand new" -s g -l global -d 'Use global ~/.forge/templates'
complete -c forge -n "__fish_forge_using_subcommand new" -l default -d 'Use default values for all prompts'
complete -c forge -n "__fish_forge_using_subcommand new" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c forge -n "__fish_forge_using_subcommand list" -s g -l global -d 'Show only global templates'
complete -c forge -n "__fish_forge_using_subcommand list" -s l -l local -d 'Show only local templates'
complete -c forge -n "__fish_forge_using_subcommand list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c forge -n "__fish_forge_using_subcommand info" -s l -l local -d 'Use local .forge/templates'
complete -c forge -n "__fish_forge_using_subcommand info" -s g -l global -d 'Use global ~/.forge/templates'
complete -c forge -n "__fish_forge_using_subcommand info" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c forge -n "__fish_forge_using_subcommand create" -s l -l local -d 'Create in local .forge/templates/'
complete -c forge -n "__fish_forge_using_subcommand create" -s g -l global -d 'Create in ~/.forge/templates/'
complete -c forge -n "__fish_forge_using_subcommand create" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c forge -n "__fish_forge_using_subcommand remove" -s l -l local -d 'Remove from local .forge/templates/'
complete -c forge -n "__fish_forge_using_subcommand remove" -s g -l global -d 'Remove from global ~/.forge/templates/'
complete -c forge -n "__fish_forge_using_subcommand remove" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c forge -n "__fish_forge_using_subcommand check" -s l -l local -d 'Check a template in local .forge/templates/'
complete -c forge -n "__fish_forge_using_subcommand check" -s g -l global -d 'Check a template in ~/.forge/templates/'
complete -c forge -n "__fish_forge_using_subcommand check" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c forge -n "__fish_forge_using_subcommand trust; and not __fish_seen_subcommand_from add remove list help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c forge -n "__fish_forge_using_subcommand trust; and not __fish_seen_subcommand_from add remove list help" -f -a "add" -d 'Trust a template and store its checksum'
complete -c forge -n "__fish_forge_using_subcommand trust; and not __fish_seen_subcommand_from add remove list help" -f -a "remove" -d 'Revoke trust from a template'
complete -c forge -n "__fish_forge_using_subcommand trust; and not __fish_seen_subcommand_from add remove list help" -f -a "list" -d 'List all trusted templates'
complete -c forge -n "__fish_forge_using_subcommand trust; and not __fish_seen_subcommand_from add remove list help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c forge -n "__fish_forge_using_subcommand trust; and __fish_seen_subcommand_from add" -s l -l local -d 'Trust a template in local .forge/templates/'
complete -c forge -n "__fish_forge_using_subcommand trust; and __fish_seen_subcommand_from add" -s g -l global -d 'Trust a template in ~/.forge/templates/'
complete -c forge -n "__fish_forge_using_subcommand trust; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c forge -n "__fish_forge_using_subcommand trust; and __fish_seen_subcommand_from remove" -s l -l local -d 'Remove trust for a template in local .forge/templates/'
complete -c forge -n "__fish_forge_using_subcommand trust; and __fish_seen_subcommand_from remove" -s g -l global -d 'Remove trust for a template in ~/.forge/templates/'
complete -c forge -n "__fish_forge_using_subcommand trust; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c forge -n "__fish_forge_using_subcommand trust; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c forge -n "__fish_forge_using_subcommand trust; and __fish_seen_subcommand_from help" -f -a "add" -d 'Trust a template and store its checksum'
complete -c forge -n "__fish_forge_using_subcommand trust; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Revoke trust from a template'
complete -c forge -n "__fish_forge_using_subcommand trust; and __fish_seen_subcommand_from help" -f -a "list" -d 'List all trusted templates'
complete -c forge -n "__fish_forge_using_subcommand trust; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c forge -n "__fish_forge_using_subcommand config; and not __fish_seen_subcommand_from set get list edit help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c forge -n "__fish_forge_using_subcommand config; and not __fish_seen_subcommand_from set get list edit help" -f -a "set" -d 'Set a config key'
complete -c forge -n "__fish_forge_using_subcommand config; and not __fish_seen_subcommand_from set get list edit help" -f -a "get" -d 'Get a config key'
complete -c forge -n "__fish_forge_using_subcommand config; and not __fish_seen_subcommand_from set get list edit help" -f -a "list" -d 'List all config keys'
complete -c forge -n "__fish_forge_using_subcommand config; and not __fish_seen_subcommand_from set get list edit help" -f -a "edit" -d 'Open config.toml in $EDITOR'
complete -c forge -n "__fish_forge_using_subcommand config; and not __fish_seen_subcommand_from set get list edit help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c forge -n "__fish_forge_using_subcommand config; and __fish_seen_subcommand_from set" -s h -l help -d 'Print help'
complete -c forge -n "__fish_forge_using_subcommand config; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c forge -n "__fish_forge_using_subcommand config; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c forge -n "__fish_forge_using_subcommand config; and __fish_seen_subcommand_from edit" -s h -l help -d 'Print help'
complete -c forge -n "__fish_forge_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "set" -d 'Set a config key'
complete -c forge -n "__fish_forge_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "get" -d 'Get a config key'
complete -c forge -n "__fish_forge_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "list" -d 'List all config keys'
complete -c forge -n "__fish_forge_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "edit" -d 'Open config.toml in $EDITOR'
complete -c forge -n "__fish_forge_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c forge -n "__fish_forge_using_subcommand package; and not __fish_seen_subcommand_from probe install remove update list help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c forge -n "__fish_forge_using_subcommand package; and not __fish_seen_subcommand_from probe install remove update list help" -f -a "probe" -d 'Probe templates from a remote repository'
complete -c forge -n "__fish_forge_using_subcommand package; and not __fish_seen_subcommand_from probe install remove update list help" -f -a "install" -d 'Install templates from a remote repository'
complete -c forge -n "__fish_forge_using_subcommand package; and not __fish_seen_subcommand_from probe install remove update list help" -f -a "remove" -d 'Remove installed template package(s)'
complete -c forge -n "__fish_forge_using_subcommand package; and not __fish_seen_subcommand_from probe install remove update list help" -f -a "update" -d 'Update installed template package(s)'
complete -c forge -n "__fish_forge_using_subcommand package; and not __fish_seen_subcommand_from probe install remove update list help" -f -a "list" -d 'List installed template packages'
complete -c forge -n "__fish_forge_using_subcommand package; and not __fish_seen_subcommand_from probe install remove update list help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c forge -n "__fish_forge_using_subcommand package; and __fish_seen_subcommand_from probe" -s h -l help -d 'Print help'
complete -c forge -n "__fish_forge_using_subcommand package; and __fish_seen_subcommand_from install" -l interactive -d 'Prompt to select template(s) when names are omitted'
complete -c forge -n "__fish_forge_using_subcommand package; and __fish_seen_subcommand_from install" -s h -l help -d 'Print help'
complete -c forge -n "__fish_forge_using_subcommand package; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help'
complete -c forge -n "__fish_forge_using_subcommand package; and __fish_seen_subcommand_from update" -s h -l help -d 'Print help'
complete -c forge -n "__fish_forge_using_subcommand package; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c forge -n "__fish_forge_using_subcommand package; and __fish_seen_subcommand_from help" -f -a "probe" -d 'Probe templates from a remote repository'
complete -c forge -n "__fish_forge_using_subcommand package; and __fish_seen_subcommand_from help" -f -a "install" -d 'Install templates from a remote repository'
complete -c forge -n "__fish_forge_using_subcommand package; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Remove installed template package(s)'
complete -c forge -n "__fish_forge_using_subcommand package; and __fish_seen_subcommand_from help" -f -a "update" -d 'Update installed template package(s)'
complete -c forge -n "__fish_forge_using_subcommand package; and __fish_seen_subcommand_from help" -f -a "list" -d 'List installed template packages'
complete -c forge -n "__fish_forge_using_subcommand package; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c forge -n "__fish_forge_using_subcommand help; and not __fish_seen_subcommand_from new list info create remove check trust config package help" -f -a "new" -d 'Scaffold a new project from a template'
complete -c forge -n "__fish_forge_using_subcommand help; and not __fish_seen_subcommand_from new list info create remove check trust config package help" -f -a "list" -d 'List available templates'
complete -c forge -n "__fish_forge_using_subcommand help; and not __fish_seen_subcommand_from new list info create remove check trust config package help" -f -a "info" -d 'Print details about a template'
complete -c forge -n "__fish_forge_using_subcommand help; and not __fish_seen_subcommand_from new list info create remove check trust config package help" -f -a "create" -d 'Scaffold a new blank template'
complete -c forge -n "__fish_forge_using_subcommand help; and not __fish_seen_subcommand_from new list info create remove check trust config package help" -f -a "remove" -d 'Remove template(s)'
complete -c forge -n "__fish_forge_using_subcommand help; and not __fish_seen_subcommand_from new list info create remove check trust config package help" -f -a "check" -d 'Check a template without executing it'
complete -c forge -n "__fish_forge_using_subcommand help; and not __fish_seen_subcommand_from new list info create remove check trust config package help" -f -a "trust" -d 'Manage template trust'
complete -c forge -n "__fish_forge_using_subcommand help; and not __fish_seen_subcommand_from new list info create remove check trust config package help" -f -a "config" -d 'Manage Forge configuration'
complete -c forge -n "__fish_forge_using_subcommand help; and not __fish_seen_subcommand_from new list info create remove check trust config package help" -f -a "package" -d 'Manage remote template packages'
complete -c forge -n "__fish_forge_using_subcommand help; and not __fish_seen_subcommand_from new list info create remove check trust config package help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c forge -n "__fish_forge_using_subcommand help; and __fish_seen_subcommand_from trust" -f -a "add" -d 'Trust a template and store its checksum'
complete -c forge -n "__fish_forge_using_subcommand help; and __fish_seen_subcommand_from trust" -f -a "remove" -d 'Revoke trust from a template'
complete -c forge -n "__fish_forge_using_subcommand help; and __fish_seen_subcommand_from trust" -f -a "list" -d 'List all trusted templates'
complete -c forge -n "__fish_forge_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "set" -d 'Set a config key'
complete -c forge -n "__fish_forge_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "get" -d 'Get a config key'
complete -c forge -n "__fish_forge_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "list" -d 'List all config keys'
complete -c forge -n "__fish_forge_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "edit" -d 'Open config.toml in $EDITOR'
complete -c forge -n "__fish_forge_using_subcommand help; and __fish_seen_subcommand_from package" -f -a "probe" -d 'Probe templates from a remote repository'
complete -c forge -n "__fish_forge_using_subcommand help; and __fish_seen_subcommand_from package" -f -a "install" -d 'Install templates from a remote repository'
complete -c forge -n "__fish_forge_using_subcommand help; and __fish_seen_subcommand_from package" -f -a "remove" -d 'Remove installed template package(s)'
complete -c forge -n "__fish_forge_using_subcommand help; and __fish_seen_subcommand_from package" -f -a "update" -d 'Update installed template package(s)'
complete -c forge -n "__fish_forge_using_subcommand help; and __fish_seen_subcommand_from package" -f -a "list" -d 'List installed template packages'
