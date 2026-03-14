# uv-shell

A fast Rust binary that creates and activates Python virtual environments using [uv](https://github.com/astral-sh/uv). Zero external crate dependencies.

## Features

- **`uv-shell`** -- Create `.venv` if missing, set a smart prompt (`<project>-py<version>`), and spawn an activated subshell
- **`uv-shell anchor`** -- Print shell commands that activate `.venv` in the current shell (for use in shell rc files)
- All `uv venv` options forwarded transparently (`--python`, `--seed`, `--clear`, `--prompt`, etc.)
- Cross-platform: Unix (exec), Windows (PowerShell + CMD)
- Shell completions for bash, zsh, fish, nushell, and PowerShell
- **`uv` plugin wrapper** (`adds-on`) -- Enables `uv shell` as a native subcommand and injects plugins into `uv <TAB>` completions

## Install

### Homebrew (macOS/Linux)

```sh
brew tap benbenbang/forge
brew install uv-shell
```

### From source

```sh
cargo install --path .
```

Or build locally:

```sh
cargo build --release
# binary at target/release/uv-shell
```

**Requirements:** [uv](https://docs.astral.sh/uv/getting-started/installation/) must be installed and on `PATH`.

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

## uv plugin wrapper (adds-on)

The `adds-on` directory contains a small Rust binary also named `uv` that acts as a plugin dispatcher. When installed before the real `uv` in `PATH`, it enables:

- `uv shell` → automatically finds and runs `uv-shell`
- `uv <any-plugin>` → finds and runs `uv-<plugin>` from PATH
- `uv <TAB>` → shows installed plugins alongside built-in uv subcommands
- `uv shell <TAB>` → shows `uv-shell`'s own options

### Setup (after `brew install uv-shell`)

**1. Add to `~/.zshrc`** (permanent, one line):

```sh
export PATH="$(brew --prefix uv-shell)/libexec/bin:$PATH"
```

**2. Install completions once:**

```sh
# zsh — write to fpath, auto-loaded on every new session
uv generate-shell-completion zsh > "${fpath[1]}/_uv"

# bash
echo 'eval "$(uv generate-shell-completion bash)"' >> ~/.bashrc

# fish
uv generate-shell-completion fish > ~/.config/fish/completions/uv.fish
```

That's it. No `eval`, no reloading. New plugins are picked up automatically on every `<TAB>`.

**Optional:** skip PATH scan on every `uv` call:
```sh
export UV_REAL_PATH="/opt/homebrew/bin/uv"
```

### How it works

```
uv shell          →  finds uv-shell in PATH  →  exec uv-shell
uv add requests   →  no uv-add plugin found  →  exec real uv add requests
uv <TAB>          →  real uv completions + discovered plugins injected
uv shell <TAB>    →  delegates to _uv-shell() from uv-shell's own completions
```

Any executable named `uv-<name>` on PATH is automatically discovered — no registration needed.

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
