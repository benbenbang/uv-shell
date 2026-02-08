_uv_shell() {
    local cur prev words cword
    _init_completion || return

    # Subcommands
    local commands="anchor completions"

    # All options
    local opts="-p --python --seed -c --clear --allow-existing --prompt
        --system-site-packages --relocatable --no-project --no-config
        --index-strategy --keyring-provider --exclude-newer --link-mode
        --index --default-index -i --index-url --extra-index-url
        -f --find-links --no-index --refresh -n --no-cache --cache-dir
        -q --quiet -v --verbose --color --native-tls --offline
        --allow-insecure-host --no-progress --directory --project
        --config-file --no-config -h --help"

    # Handle completions for options that take a value
    case "$prev" in
        -p|--python|--prompt|--index-strategy|--keyring-provider| \
        --link-mode|--exclude-newer|--exclude-newer-package| \
        --index|--default-index|-i|--index-url|--extra-index-url| \
        -f|--find-links|--cache-dir|--color|--directory|--project| \
        --config-file|--allow-insecure-host|--refresh-package)
            # These expect a value; let the user type freely
            return 0
            ;;
        completions)
            COMPREPLY=($(compgen -W "bash zsh fish nushell powershell" -- "$cur"))
            return 0
            ;;
    esac

    # anchor subcommand: offer --shell and its values
    if [[ ${words[1]} == "anchor" ]]; then
        if [[ "$prev" == "--shell" ]]; then
            COMPREPLY=($(compgen -W "bash fish nushell powershell cmd" -- "$cur"))
            return 0
        fi
        COMPREPLY=($(compgen -W "--shell" -- "$cur"))
        return 0
    fi

    # First argument: offer subcommands and options
    if [[ $cword -eq 1 ]]; then
        if [[ "$cur" == -* ]]; then
            COMPREPLY=($(compgen -W "$opts" -- "$cur"))
        else
            COMPREPLY=($(compgen -W "$commands $opts" -- "$cur"))
        fi
        return 0
    fi

    # Subsequent arguments: offer options
    if [[ "$cur" == -* ]]; then
        COMPREPLY=($(compgen -W "$opts" -- "$cur"))
    fi
} && complete -F _uv_shell uv-shell
