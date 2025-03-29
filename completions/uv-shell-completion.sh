# Check if uv is available
if type uv &>/dev/null; then
    # Generate shell completions based on platform and shell
    if [[ "$SHELL" == *"zsh"* ]]; then
        eval "$(uv generate-shell-completion zsh)"

        # Define custom completion function for zsh
        _uv_shell() {
            # Call the original _uv completion function
            _uv "$@"

            if [[ $CURRENT -eq 2 ]]; then
                # Add 'shell' to the completion options
                _values 'command' 'shell[activate or create and activate a virtual environment]'
            fi
        }
        compdef _uv_shell uv
    elif [[ "$SHELL" == *"bash"* ]]; then
        eval "$(uv generate-shell-completion bash)"

        # For bash, extend the completion in bash-compatible way
        _uv_complete_original=$(complete -p uv 2>/dev/null || echo "")
        if [[ -n "$_uv_complete_original" ]]; then
            # Extract the completion function name
            local complete_func=$(echo "$_uv_complete_original" | sed -E 's/.*-F ([^ ]+).*/\1/')

            # Define a wrapper function
            eval "$complete_func"_with_shell'() {
                local cur prev words cword
                _init_completion || return

                # Call original completion
                '"$complete_func"' "$@"

                # Add shell command if completing the main command
                if [[ ${#words[@]} -eq 2 ]]; then
                    COMPREPLY+=($(compgen -W "shell" -- "$cur"))
                fi

                # Provide description if possible in bash
                if [[ "$cur" == "shell" ]]; then
                    COMPREPLY=( "shell   -- activate or create and activate a virtual environment" )
                fi
            }'

            # Register the wrapper function
            complete -F "$complete_func"_with_shell uv
        fi
    elif [[ "$SHELL" == *"fish"* ]]; then
        # For fish shell, the approach would be different
        # Fish completions are typically stored in files
        if [[ ! -d ~/.config/fish/completions ]]; then
            mkdir -p ~/.config/fish/completions
        fi

        # Generate fish completions if needed
        if [[ ! -f ~/.config/fish/completions/uv.fish ]]; then
            uv generate-shell-completion fish > ~/.config/fish/completions/uv.fish
        fi

        # Add shell command to fish completions
        if ! grep -q "shell.*activate or create" ~/.config/fish/completions/uv.fish; then
            echo "complete -c uv -n '__fish_use_subcommand' -a shell -d 'activate or create and activate a virtual environment'" >> ~/.config/fish/completions/uv.fish
        fi
    elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" || "$OSTYPE" == "cygwin" ]]; then
        # For Windows PowerShell, we'd use a different approach
        # This is just a placeholder as PowerShell completions work differently
        if [[ "$SHELL" == *"powershell"* || "$SHELL" == *"pwsh"* ]]; then
            # PowerShell would need a module approach
            echo "PowerShell completions for uv require a separate module setup"
        fi
    fi
fi
