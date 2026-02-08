# uv-shell

A fast Rust binary that creates and activates Python virtual environments using [uv](https://github.com/astral-sh/uv). Zero external crate dependencies.

## Features

- **`uv-shell`** -- Create `.venv` if missing, set a smart prompt (`<project>-py<version>`), and spawn an activated subshell
- **`uv-shell anchor`** -- Print shell commands that activate `.venv` in the current shell (for use in shell rc files)
- All `uv venv` options forwarded transparently (`--python`, `--seed`, `--clear`, `--prompt`, etc.)
- Cross-platform: Unix (exec), Windows (PowerShell + CMD)
- Shell completions for bash, zsh, fish, nushell, and PowerShell

## Install

```sh
cargo install --path .
```

Or build locally:

```sh
cargo build --release
# binary at target/release/uv-shell
```

Requires [uv](https://docs.astral.sh/uv/getting-started/installation/) to be installed and on `PATH`.

## Usage

### Create and activate a virtual environment

```sh
# Create .venv (if missing) and spawn an activated subshell
uv-shell

# With a specific Python version
uv-shell -p 3.12

# With seed packages (pip, setuptools, wheel)
uv-shell --seed

# Re-create an existing venv
uv-shell --clear

# Custom prompt instead of auto-generated <project>-py<version>
uv-shell --prompt my-env
```

All options are forwarded to `uv venv`. Run `uv-shell --help` or `uv venv --help` for the full list.

To exit the activated subshell, press `Ctrl + D`.

### Auto-activate with anchor

Add to your shell rc file to auto-activate `.venv` when it exists in the current directory:

**bash / zsh** (`~/.bashrc` or `~/.zshrc`):
```sh
eval "$(uv-shell anchor)"
```

**fish** (`~/.config/fish/config.fish`):
```fish
uv-shell anchor | source
```

**PowerShell** (`$PROFILE`):
```powershell
uv-shell anchor | Invoke-Expression
```

**nushell** (`config.nu`):
```nu
# save to a file and source it
uv-shell anchor --shell nushell | save -f ~/.venv-anchor.nu
source ~/.venv-anchor.nu
```

The anchor command auto-detects your shell. Override with `--shell`:

```sh
uv-shell anchor --shell powershell
uv-shell anchor --shell fish
uv-shell anchor --shell cmd
```

### Shell completions

```sh
# bash
eval "$(uv-shell completions bash)"

# zsh (add to fpath)
uv-shell completions zsh > "${fpath[1]}/_uv-shell"

# fish
uv-shell completions fish > ~/.config/fish/completions/uv-shell.fish

# nushell
uv-shell completions nushell | save -f ~/.config/nushell/uv-shell-completions.nu

# PowerShell (add to $PROFILE)
uv-shell completions powershell >> $PROFILE
```

## How it works

| Step | What happens |
|---|---|
| 1. Resolve path | Canonicalize `.venv` relative to CWD |
| 2. Create venv | Run `uv venv <path>` with forwarded options (if `.venv` missing or `--clear`) |
| 3. Update prompt | Patch `pyvenv.cfg` with `prompt = <project>-py<version>` (skipped if `--prompt` given) |
| 4. Activate | **Unix**: `exec $SHELL` with `VIRTUAL_ENV` and `PATH` set (replaces process, no nesting) / **Windows**: spawn PowerShell or CMD with env vars |

The `anchor` command prints shell-appropriate commands to set `VIRTUAL_ENV`, `PIP_REQUIRE_VIRTUALENV`, and prepend the venv `bin`/`Scripts` directory to `PATH`.

## License

[MIT](LICENSE)
