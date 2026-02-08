#compdef uv-shell

_uv-shell() {
    local -a commands
    commands=(
        'anchor:Print export commands for shell rc eval'
        'completions:Generate shell completions'
    )

    local -a uv_venv_opts
    uv_venv_opts=(
        '(-p --python)'{-p,--python}'[Python interpreter to use]:python:'
        '--seed[Install seed packages (pip, setuptools, wheel)]'
        '(-c --clear)'{-c,--clear}'[Re-create the venv even if .venv already exists]'
        '--allow-existing[Preserve existing files at the target path]'
        '--prompt[Custom prompt prefix (skips auto-prompt)]:prompt:'
        '--system-site-packages[Give the venv access to system site packages]'
        '--relocatable[Make the venv relocatable]'
        '--no-project[Avoid discovering a project or workspace]'
        '--index-strategy[Strategy for resolving against multiple indexes]:strategy:(first-index unsafe-first-match unsafe-best-match)'
        '--keyring-provider[Keyring provider for authentication]:provider:(disabled subprocess)'
        '--exclude-newer[Limit packages to those uploaded before date]:date:'
        '--link-mode[Method for installing packages from cache]:mode:(clone copy hardlink symlink)'
        '--index[Additional index URLs]:url:'
        '--default-index[Default package index URL]:url:'
        '(-f --find-links)'{-f,--find-links}'[Locations to search for distributions]:path:_files'
        '--no-index[Ignore the registry index]'
        '--refresh[Refresh all cached data]'
        '(-n --no-cache)'{-n,--no-cache}'[Avoid reading from or writing to the cache]'
        '--cache-dir[Path to the cache directory]:dir:_files -/'
        '(-q --quiet)'{-q,--quiet}'[Quiet output]'
        '(-v --verbose)'{-v,--verbose}'[Verbose output]'
        '--color[Control use of color]:when:(auto always never)'
        '--native-tls[Load TLS certificates from platform native store]'
        '--offline[Disable network access]'
        '--allow-insecure-host[Allow insecure connections to a host]:host:'
        '--no-progress[Hide all progress outputs]'
        '--directory[Change to given directory before running]:dir:_files -/'
        '--project[Discover a project in given directory]:dir:_files -/'
        '--config-file[Path to uv.toml]:file:_files'
        '--no-config[Avoid discovering configuration files]'
        '(-h --help)'{-h,--help}'[Show help message]'
    )

    _arguments -s -S \
        '1: :->cmd' \
        '*:: :->args'

    case $state in
        cmd)
            _describe 'command' commands
            _arguments -s -S $uv_venv_opts
            ;;
        args)
            case $words[1] in
                completions)
                    _values 'shell' bash zsh fish nushell powershell
                    ;;
                anchor)
                    _arguments -s -S \
                        '--shell[Override shell detection]:shell:(bash fish nushell powershell cmd)'
                    ;;
                *)
                    _arguments -s -S $uv_venv_opts
                    ;;
            esac
            ;;
    esac
}

_uv-shell "$@"
