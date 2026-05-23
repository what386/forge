
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'forge' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'forge'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'forge' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('new', 'new', [CompletionResultType]::ParameterValue, 'Scaffold a new project from a template')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List available templates')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Print details about a template')
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Scaffold a new blank template')
            [CompletionResult]::new('check', 'check', [CompletionResultType]::ParameterValue, 'Check a template without executing it')
            [CompletionResult]::new('trust', 'trust', [CompletionResultType]::ParameterValue, 'Manage template trust')
            [CompletionResult]::new('config', 'config', [CompletionResultType]::ParameterValue, 'Manage Forge configuration')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'forge;new' {
            [CompletionResult]::new('--default', '--default', [CompletionResultType]::ParameterName, 'Use default values for all prompts')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'forge;list' {
            [CompletionResult]::new('-g', '-g', [CompletionResultType]::ParameterName, 'Show only global templates')
            [CompletionResult]::new('--global', '--global', [CompletionResultType]::ParameterName, 'Show only global templates')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'Show only local templates')
            [CompletionResult]::new('--local', '--local', [CompletionResultType]::ParameterName, 'Show only local templates')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'forge;info' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'forge;create' {
            [CompletionResult]::new('-g', '-g', [CompletionResultType]::ParameterName, 'Create in ~/.forge/templates/ instead of .forge/templates/')
            [CompletionResult]::new('--global', '--global', [CompletionResultType]::ParameterName, 'Create in ~/.forge/templates/ instead of .forge/templates/')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'forge;check' {
            [CompletionResult]::new('-g', '-g', [CompletionResultType]::ParameterName, 'Check a template in ~/.forge/templates/')
            [CompletionResult]::new('--global', '--global', [CompletionResultType]::ParameterName, 'Check a template in ~/.forge/templates/')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'forge;trust' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Trust a template and store its checksum')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Revoke trust from a template')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all trusted templates')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'forge;trust;add' {
            [CompletionResult]::new('-g', '-g', [CompletionResultType]::ParameterName, 'Trust a template in ~/.forge/templates/')
            [CompletionResult]::new('--global', '--global', [CompletionResultType]::ParameterName, 'Trust a template in ~/.forge/templates/')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'forge;trust;remove' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'forge;trust;list' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'forge;trust;help' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Trust a template and store its checksum')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Revoke trust from a template')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all trusted templates')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'forge;trust;help;add' {
            break
        }
        'forge;trust;help;remove' {
            break
        }
        'forge;trust;help;list' {
            break
        }
        'forge;trust;help;help' {
            break
        }
        'forge;config' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('set', 'set', [CompletionResultType]::ParameterValue, 'Set a config key')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get a config key')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all config keys')
            [CompletionResult]::new('edit', 'edit', [CompletionResultType]::ParameterValue, 'Open config.toml in $EDITOR')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'forge;config;set' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'forge;config;get' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'forge;config;list' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'forge;config;edit' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'forge;config;help' {
            [CompletionResult]::new('set', 'set', [CompletionResultType]::ParameterValue, 'Set a config key')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get a config key')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all config keys')
            [CompletionResult]::new('edit', 'edit', [CompletionResultType]::ParameterValue, 'Open config.toml in $EDITOR')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'forge;config;help;set' {
            break
        }
        'forge;config;help;get' {
            break
        }
        'forge;config;help;list' {
            break
        }
        'forge;config;help;edit' {
            break
        }
        'forge;config;help;help' {
            break
        }
        'forge;help' {
            [CompletionResult]::new('new', 'new', [CompletionResultType]::ParameterValue, 'Scaffold a new project from a template')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List available templates')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Print details about a template')
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Scaffold a new blank template')
            [CompletionResult]::new('check', 'check', [CompletionResultType]::ParameterValue, 'Check a template without executing it')
            [CompletionResult]::new('trust', 'trust', [CompletionResultType]::ParameterValue, 'Manage template trust')
            [CompletionResult]::new('config', 'config', [CompletionResultType]::ParameterValue, 'Manage Forge configuration')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'forge;help;new' {
            break
        }
        'forge;help;list' {
            break
        }
        'forge;help;info' {
            break
        }
        'forge;help;create' {
            break
        }
        'forge;help;check' {
            break
        }
        'forge;help;trust' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Trust a template and store its checksum')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Revoke trust from a template')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all trusted templates')
            break
        }
        'forge;help;trust;add' {
            break
        }
        'forge;help;trust;remove' {
            break
        }
        'forge;help;trust;list' {
            break
        }
        'forge;help;config' {
            [CompletionResult]::new('set', 'set', [CompletionResultType]::ParameterValue, 'Set a config key')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get a config key')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all config keys')
            [CompletionResult]::new('edit', 'edit', [CompletionResultType]::ParameterValue, 'Open config.toml in $EDITOR')
            break
        }
        'forge;help;config;set' {
            break
        }
        'forge;help;config;get' {
            break
        }
        'forge;help;config;list' {
            break
        }
        'forge;help;config;edit' {
            break
        }
        'forge;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
