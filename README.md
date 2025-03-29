# UV Shell

A shell wrapper for [UV](https://github.com/astral-sh/uv) that provides enhanced virtual environment management.

## Features

- Simplified virtual environment creation and activation with a single command
- Automatic environment naming based on project directory and Python version
- Cross-platform support for macOS, Linux, and Windows
- Support for multiple shells (bash, zsh, fish, PowerShell)

## Key Points, Limitations, and Manual Setup
Essentially, the way that virtual environment prompt get updated `(project_name-python_version)` is very simple.
It just needs to add the following line to your `./.venv/pyvenv.cfg` file:
```
prompt = "({project_name}-{python_version})"
```
This is a manual setup, but it's very simple and works well.

This repository is for the `uv-shell.sh` script, which aims for a more automated setup.

## Installation

1. Ensure you have [uv](https://github.com/astral-sh/uv) installed
2. Clone this repository or download `uv-shell.sh`
3. Add it to your shell configuration:

```bash
# In your .bashrc, .zshrc, or equivalent:
source /path/to/uv-shell.sh
```

## Usage

Run the `uv shell` command in any project directory:

```bash
$ cd your-project
$ uv shell
```

This will:
1. Create a virtual environment (`.venv`) if it doesn't exist
2. Set the prompt to include the project name and Python version
3. Activate the environment in a new shell session

To exit the virtual environment, press `Ctrl+D` or type `exit`.

## How It Works

The script implements a wrapper around the standard `uv` command that:
- Intercepts the `shell` subcommand to provide enhanced functionality
- Passes all other subcommands directly to the original `uv` command
- Handles platform-specific configuration for different operating systems and shells

## License

See the LICENSE file for details.
