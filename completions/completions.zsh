#compdef forge

autoload -U is-at-least

_forge() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
":: :_forge_commands" \
"*::: :->forge" \
&& ret=0
    case $state in
    (forge)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:forge-command-$line[1]:"
        case $line[1] in
            (new)
_arguments "${_arguments_options[@]}" : \
'--default[Use default values for all prompts]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':template -- Name of the template to use:_default' \
':name -- Name of the project to create:_default' \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
'(-l --local)-g[Show only global templates]' \
'(-l --local)--global[Show only global templates]' \
'(-g --global)-l[Show only local templates]' \
'(-g --global)--local[Show only local templates]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(info)
_arguments "${_arguments_options[@]}" : \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':template -- Name of the template to inspect:_default' \
&& ret=0
;;
(create)
_arguments "${_arguments_options[@]}" : \
'-g[Create in ~/.forge/templates/ instead of .forge/templates/]' \
'--global[Create in ~/.forge/templates/ instead of .forge/templates/]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':name -- Name of the template to create:_default' \
&& ret=0
;;
(check)
_arguments "${_arguments_options[@]}" : \
'-g[Check a template in ~/.forge/templates/]' \
'--global[Check a template in ~/.forge/templates/]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':template -- Name of the template to check:_default' \
&& ret=0
;;
(trust)
_arguments "${_arguments_options[@]}" : \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_forge__subcmd__trust_commands" \
"*::: :->trust" \
&& ret=0

    case $state in
    (trust)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:forge-trust-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
'-g[Trust a template in ~/.forge/templates/]' \
'--global[Trust a template in ~/.forge/templates/]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':template -- Name of the template to trust:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':template -- Name of the template to untrust:_default' \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_forge__subcmd__trust__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:forge-trust-help-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(config)
_arguments "${_arguments_options[@]}" : \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_forge__subcmd__config_commands" \
"*::: :->config" \
&& ret=0

    case $state in
    (config)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:forge-config-command-$line[1]:"
        case $line[1] in
            (set)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':key -- Dot-separated config key path:_default' \
':value -- Config value (TOML literal or plain string):_default' \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':key -- Dot-separated config key path:_default' \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(edit)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_forge__subcmd__config__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:forge-config-help-command-$line[1]:"
        case $line[1] in
            (set)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(edit)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_forge__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:forge-help-command-$line[1]:"
        case $line[1] in
            (new)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(info)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(create)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(check)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(trust)
_arguments "${_arguments_options[@]}" : \
":: :_forge__subcmd__help__subcmd__trust_commands" \
"*::: :->trust" \
&& ret=0

    case $state in
    (trust)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:forge-help-trust-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(config)
_arguments "${_arguments_options[@]}" : \
":: :_forge__subcmd__help__subcmd__config_commands" \
"*::: :->config" \
&& ret=0

    case $state in
    (config)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:forge-help-config-command-$line[1]:"
        case $line[1] in
            (set)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(edit)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
}

(( $+functions[_forge_commands] )) ||
_forge_commands() {
    local commands; commands=(
'new:Scaffold a new project from a template' \
'list:List available templates' \
'info:Print details about a template' \
'create:Scaffold a new blank template' \
'check:Check a template without executing it' \
'trust:Manage template trust' \
'config:Manage Forge configuration' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'forge commands' commands "$@"
}
(( $+functions[_forge__subcmd__check_commands] )) ||
_forge__subcmd__check_commands() {
    local commands; commands=()
    _describe -t commands 'forge check commands' commands "$@"
}
(( $+functions[_forge__subcmd__config_commands] )) ||
_forge__subcmd__config_commands() {
    local commands; commands=(
'set:Set a config key' \
'get:Get a config key' \
'list:List all config keys' \
'edit:Open config.toml in \$EDITOR' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'forge config commands' commands "$@"
}
(( $+functions[_forge__subcmd__config__subcmd__edit_commands] )) ||
_forge__subcmd__config__subcmd__edit_commands() {
    local commands; commands=()
    _describe -t commands 'forge config edit commands' commands "$@"
}
(( $+functions[_forge__subcmd__config__subcmd__get_commands] )) ||
_forge__subcmd__config__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'forge config get commands' commands "$@"
}
(( $+functions[_forge__subcmd__config__subcmd__help_commands] )) ||
_forge__subcmd__config__subcmd__help_commands() {
    local commands; commands=(
'set:Set a config key' \
'get:Get a config key' \
'list:List all config keys' \
'edit:Open config.toml in \$EDITOR' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'forge config help commands' commands "$@"
}
(( $+functions[_forge__subcmd__config__subcmd__help__subcmd__edit_commands] )) ||
_forge__subcmd__config__subcmd__help__subcmd__edit_commands() {
    local commands; commands=()
    _describe -t commands 'forge config help edit commands' commands "$@"
}
(( $+functions[_forge__subcmd__config__subcmd__help__subcmd__get_commands] )) ||
_forge__subcmd__config__subcmd__help__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'forge config help get commands' commands "$@"
}
(( $+functions[_forge__subcmd__config__subcmd__help__subcmd__help_commands] )) ||
_forge__subcmd__config__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'forge config help help commands' commands "$@"
}
(( $+functions[_forge__subcmd__config__subcmd__help__subcmd__list_commands] )) ||
_forge__subcmd__config__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'forge config help list commands' commands "$@"
}
(( $+functions[_forge__subcmd__config__subcmd__help__subcmd__set_commands] )) ||
_forge__subcmd__config__subcmd__help__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'forge config help set commands' commands "$@"
}
(( $+functions[_forge__subcmd__config__subcmd__list_commands] )) ||
_forge__subcmd__config__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'forge config list commands' commands "$@"
}
(( $+functions[_forge__subcmd__config__subcmd__set_commands] )) ||
_forge__subcmd__config__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'forge config set commands' commands "$@"
}
(( $+functions[_forge__subcmd__create_commands] )) ||
_forge__subcmd__create_commands() {
    local commands; commands=()
    _describe -t commands 'forge create commands' commands "$@"
}
(( $+functions[_forge__subcmd__help_commands] )) ||
_forge__subcmd__help_commands() {
    local commands; commands=(
'new:Scaffold a new project from a template' \
'list:List available templates' \
'info:Print details about a template' \
'create:Scaffold a new blank template' \
'check:Check a template without executing it' \
'trust:Manage template trust' \
'config:Manage Forge configuration' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'forge help commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__check_commands] )) ||
_forge__subcmd__help__subcmd__check_commands() {
    local commands; commands=()
    _describe -t commands 'forge help check commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__config_commands] )) ||
_forge__subcmd__help__subcmd__config_commands() {
    local commands; commands=(
'set:Set a config key' \
'get:Get a config key' \
'list:List all config keys' \
'edit:Open config.toml in \$EDITOR' \
    )
    _describe -t commands 'forge help config commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__config__subcmd__edit_commands] )) ||
_forge__subcmd__help__subcmd__config__subcmd__edit_commands() {
    local commands; commands=()
    _describe -t commands 'forge help config edit commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__config__subcmd__get_commands] )) ||
_forge__subcmd__help__subcmd__config__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'forge help config get commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__config__subcmd__list_commands] )) ||
_forge__subcmd__help__subcmd__config__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'forge help config list commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__config__subcmd__set_commands] )) ||
_forge__subcmd__help__subcmd__config__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'forge help config set commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__create_commands] )) ||
_forge__subcmd__help__subcmd__create_commands() {
    local commands; commands=()
    _describe -t commands 'forge help create commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__help_commands] )) ||
_forge__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'forge help help commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__info_commands] )) ||
_forge__subcmd__help__subcmd__info_commands() {
    local commands; commands=()
    _describe -t commands 'forge help info commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__list_commands] )) ||
_forge__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'forge help list commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__new_commands] )) ||
_forge__subcmd__help__subcmd__new_commands() {
    local commands; commands=()
    _describe -t commands 'forge help new commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__trust_commands] )) ||
_forge__subcmd__help__subcmd__trust_commands() {
    local commands; commands=(
'add:Trust a template and store its checksum' \
'remove:Revoke trust from a template' \
'list:List all trusted templates' \
    )
    _describe -t commands 'forge help trust commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__trust__subcmd__add_commands] )) ||
_forge__subcmd__help__subcmd__trust__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'forge help trust add commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__trust__subcmd__list_commands] )) ||
_forge__subcmd__help__subcmd__trust__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'forge help trust list commands' commands "$@"
}
(( $+functions[_forge__subcmd__help__subcmd__trust__subcmd__remove_commands] )) ||
_forge__subcmd__help__subcmd__trust__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'forge help trust remove commands' commands "$@"
}
(( $+functions[_forge__subcmd__info_commands] )) ||
_forge__subcmd__info_commands() {
    local commands; commands=()
    _describe -t commands 'forge info commands' commands "$@"
}
(( $+functions[_forge__subcmd__list_commands] )) ||
_forge__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'forge list commands' commands "$@"
}
(( $+functions[_forge__subcmd__new_commands] )) ||
_forge__subcmd__new_commands() {
    local commands; commands=()
    _describe -t commands 'forge new commands' commands "$@"
}
(( $+functions[_forge__subcmd__trust_commands] )) ||
_forge__subcmd__trust_commands() {
    local commands; commands=(
'add:Trust a template and store its checksum' \
'remove:Revoke trust from a template' \
'list:List all trusted templates' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'forge trust commands' commands "$@"
}
(( $+functions[_forge__subcmd__trust__subcmd__add_commands] )) ||
_forge__subcmd__trust__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'forge trust add commands' commands "$@"
}
(( $+functions[_forge__subcmd__trust__subcmd__help_commands] )) ||
_forge__subcmd__trust__subcmd__help_commands() {
    local commands; commands=(
'add:Trust a template and store its checksum' \
'remove:Revoke trust from a template' \
'list:List all trusted templates' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'forge trust help commands' commands "$@"
}
(( $+functions[_forge__subcmd__trust__subcmd__help__subcmd__add_commands] )) ||
_forge__subcmd__trust__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'forge trust help add commands' commands "$@"
}
(( $+functions[_forge__subcmd__trust__subcmd__help__subcmd__help_commands] )) ||
_forge__subcmd__trust__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'forge trust help help commands' commands "$@"
}
(( $+functions[_forge__subcmd__trust__subcmd__help__subcmd__list_commands] )) ||
_forge__subcmd__trust__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'forge trust help list commands' commands "$@"
}
(( $+functions[_forge__subcmd__trust__subcmd__help__subcmd__remove_commands] )) ||
_forge__subcmd__trust__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'forge trust help remove commands' commands "$@"
}
(( $+functions[_forge__subcmd__trust__subcmd__list_commands] )) ||
_forge__subcmd__trust__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'forge trust list commands' commands "$@"
}
(( $+functions[_forge__subcmd__trust__subcmd__remove_commands] )) ||
_forge__subcmd__trust__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'forge trust remove commands' commands "$@"
}

if [ "$funcstack[1]" = "_forge" ]; then
    _forge "$@"
else
    compdef _forge forge
fi
