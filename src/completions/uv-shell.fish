# Subcommands
complete -c uv-shell -n '__fish_use_subcommand' -a anchor -d 'Print export commands for shell rc eval'
complete -c uv-shell -n '__fish_use_subcommand' -a completions -d 'Generate shell completions'

# completions subcommand
complete -c uv-shell -n '__fish_seen_subcommand_from completions' -a 'bash zsh fish nushell powershell'

# anchor subcommand
complete -c uv-shell -n '__fish_seen_subcommand_from anchor' -l shell -d 'Override shell detection' -r -a 'bash fish nushell powershell cmd'

# Options (forwarded to uv venv)
complete -c uv-shell -n '__fish_use_subcommand' -s p -l python -d 'Python interpreter to use' -r
complete -c uv-shell -n '__fish_use_subcommand' -l seed -d 'Install seed packages (pip, setuptools, wheel)'
complete -c uv-shell -n '__fish_use_subcommand' -s c -l clear -d 'Re-create the venv even if .venv already exists'
complete -c uv-shell -n '__fish_use_subcommand' -l allow-existing -d 'Preserve existing files at the target path'
complete -c uv-shell -n '__fish_use_subcommand' -l prompt -d 'Custom prompt prefix (skips auto-prompt)' -r
complete -c uv-shell -n '__fish_use_subcommand' -l system-site-packages -d 'Access system site packages'
complete -c uv-shell -n '__fish_use_subcommand' -l relocatable -d 'Make the venv relocatable'
complete -c uv-shell -n '__fish_use_subcommand' -l no-project -d 'Avoid discovering a project or workspace'
complete -c uv-shell -n '__fish_use_subcommand' -l no-config -d 'Avoid discovering configuration files'
complete -c uv-shell -n '__fish_use_subcommand' -l index-strategy -d 'Strategy for resolving against multiple indexes' -r -a 'first-index unsafe-first-match unsafe-best-match'
complete -c uv-shell -n '__fish_use_subcommand' -l keyring-provider -d 'Keyring provider for authentication' -r -a 'disabled subprocess'
complete -c uv-shell -n '__fish_use_subcommand' -l exclude-newer -d 'Limit packages to those uploaded before date' -r
complete -c uv-shell -n '__fish_use_subcommand' -l link-mode -d 'Method for installing from cache' -r -a 'clone copy hardlink symlink'
complete -c uv-shell -n '__fish_use_subcommand' -l index -d 'Additional index URLs' -r
complete -c uv-shell -n '__fish_use_subcommand' -l default-index -d 'Default package index URL' -r
complete -c uv-shell -n '__fish_use_subcommand' -s f -l find-links -d 'Locations to search for distributions' -r
complete -c uv-shell -n '__fish_use_subcommand' -l no-index -d 'Ignore the registry index'
complete -c uv-shell -n '__fish_use_subcommand' -l refresh -d 'Refresh all cached data'
complete -c uv-shell -n '__fish_use_subcommand' -s n -l no-cache -d 'Avoid reading from or writing to the cache'
complete -c uv-shell -n '__fish_use_subcommand' -l cache-dir -d 'Path to the cache directory' -r -F
complete -c uv-shell -n '__fish_use_subcommand' -s q -l quiet -d 'Quiet output'
complete -c uv-shell -n '__fish_use_subcommand' -s v -l verbose -d 'Verbose output'
complete -c uv-shell -n '__fish_use_subcommand' -l color -d 'Control use of color' -r -a 'auto always never'
complete -c uv-shell -n '__fish_use_subcommand' -l native-tls -d 'Load TLS certificates from platform native store'
complete -c uv-shell -n '__fish_use_subcommand' -l offline -d 'Disable network access'
complete -c uv-shell -n '__fish_use_subcommand' -l allow-insecure-host -d 'Allow insecure connections to a host' -r
complete -c uv-shell -n '__fish_use_subcommand' -l no-progress -d 'Hide all progress outputs'
complete -c uv-shell -n '__fish_use_subcommand' -l directory -d 'Change to given directory before running' -r -a '(__fish_complete_directories)'
complete -c uv-shell -n '__fish_use_subcommand' -l project -d 'Discover a project in given directory' -r -a '(__fish_complete_directories)'
complete -c uv-shell -n '__fish_use_subcommand' -l config-file -d 'Path to uv.toml' -r -F
complete -c uv-shell -n '__fish_use_subcommand' -s h -l help -d 'Show help message'
