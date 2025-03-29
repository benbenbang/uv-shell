# UV Shell Completions

This directory contains shell completion scripts for the UV Shell wrapper.

> **Note:** These completion scripts have not been thoroughly tested yet. You may encounter issues with some shells or configurations.

## Overview

The completion scripts enable tab completion for the `uv shell` command in various shells:

- Bash
- Zsh
- Fish
- PowerShell (partial support)

## Installation

The completions are automatically loaded when you source the main `uv-shell.sh` script in your shell configuration file.

## How It Works

The completion script:

1. Detects your current shell
2. Leverages UV's built-in completion generation capabilities
3. Extends those completions to add support for the custom `shell` subcommand

## Manual Installation

If you want to install the completions separately:

### Bash

```bash
source /path/to/uv-shell/completions/uv-shell-completion.sh
```

### Zsh

```zsh
source /path/to/uv-shell/completions/uv-shell-completion.sh
```

### Fish

```fish
source /path/to/uv-shell/completions/uv-shell-completion.sh
```

## PowerShell

PowerShell completions require additional setup as they use a module-based approach. Basic support is included in the completion script, but advanced functionality may require customization.
