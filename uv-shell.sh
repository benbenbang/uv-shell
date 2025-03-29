#!/usr/bin/env bash
# ==============================================================================
#    __  ____   __    _____ __  __________    __
#   / / / / /  / /   / ___// / / / ____/ /   / /
#  / / / / /  / /    \__ \/ /_/ / __/ / /   / /
# / /_/ / /__/ /___ ___/ / __  / /___/ /___/ /
# \____/_____/_____//____/_/ /_/_____/_____/_/
#
#         🐍 VIRTUAL ENVIRONMENT WIZARD 🧙‍♂️
# ==============================================================================
#
# A magical UV wrapper for all your virtual env needs
# with extra sprinkles of automation and convenience!
#
# Creator: benbenbang <bn@benbenbang.io>
# Repo: https://github.com/benbenbang/uv-shell
# License: Do What You Want License (The one of the most permissive licenses)
#
# Legend says if you use this script enough times,
# a Python will appear and grant you three wishes...
# (Results may vary. Python appearances not guaranteed.)
# ==============================================================================

# Helper function to detect platform and set bin directory
function _uv_get_bin_dir() {
    if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" || "$OSTYPE" == "cygwin" ]]; then
        echo "Scripts"
    else
        echo "bin"
    fi
}

# Helper function to update prompt in pyvenv.cfg
function _uv_update_prompt() {
    local venv_path="$1"
    local prompt_value="$2"

    if ! grep -q "^prompt = " "$venv_path/pyvenv.cfg"; then
        # Add the prompt line if it doesn't exist
        echo "prompt = $prompt_value" >> "$venv_path/pyvenv.cfg"
    elif ! grep -q "^prompt = $prompt_value$" "$venv_path/pyvenv.cfg"; then
        # Update the prompt if it exists but with a different value
        if [[ "$OSTYPE" == "darwin"* || "$OSTYPE" == "linux-gnu"* ]]; then
            sed -i.bak "s/^prompt = .*$/prompt = $prompt_value/" "$venv_path/pyvenv.cfg"
            rm -f "$venv_path/pyvenv.cfg.bak"
        else
            # Windows-compatible sed replacement
            sed "s/^prompt = .*$/prompt = $prompt_value/" "$venv_path/pyvenv.cfg" > "$venv_path/pyvenv.cfg.new"
            mv "$venv_path/pyvenv.cfg.new" "$venv_path/pyvenv.cfg"
        fi
    fi
}

# Helper function to activate the virtual environment
function _uv_activate_venv() {
    local venv_path="$1"
    local bin_dir="$2"

    echo "Activate existing virtual environment at $venv_path"
    echo "To deactivate, hit Ctrl + D"

    if [[ "$SHELL" == *"zsh"* ]]; then
        # For zsh
        $SHELL -c "emulate bash -c '. $venv_path/$bin_dir/activate'; exec $SHELL"
    elif [[ "$SHELL" == *"fish"* ]]; then
        # For fish shell
        $SHELL -c "source $venv_path/$bin_dir/activate.fish; exec $SHELL"
    elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" || "$OSTYPE" == "cygwin" ]]; then
        # For Windows shells
        if [[ "$SHELL" == *"powershell"* || "$SHELL" == *"pwsh"* ]]; then
            # PowerShell
            pwsh -NoExit -Command ". $venv_path/$bin_dir/Activate.ps1"
        else
            # CMD
            cmd /K "$venv_path/$bin_dir/activate.bat"
        fi
    else
        # For bash and others
        $SHELL -c "source $venv_path/$bin_dir/activate; exec $SHELL"
    fi
}

# Main uv function
function uv() {
    if [[ "$1" == "shell" ]]; then
        local venv_path=".venv"
        local project_name=${PWD##*/}  # Extract current directory name

        # Create venv if needed
        if [[ ! -d "$venv_path" ]]; then
            command uv venv "$venv_path"
        fi

        # Get bin directory based on platform
        local bin_dir=$(_uv_get_bin_dir)

        if [[ -f "$venv_path/pyvenv.cfg" ]]; then
            # Get Python version
            local py_version=$($venv_path/$bin_dir/python -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')")
            local prompt_value="$project_name-py$py_version"

            # Update prompt in pyvenv.cfg
            _uv_update_prompt "$venv_path" "$prompt_value"
        fi

        # Check if .venv directory exists after creation
        if [[ -d "$venv_path" ]]; then
            # Activate the virtual environment
            _uv_activate_venv "$venv_path" "$bin_dir"
        else
            echo "Failed to create virtual environment at $venv_path"
        fi
    else
        # Forward all other commands to the actual uv command
        command uv "$@"
    fi
}
