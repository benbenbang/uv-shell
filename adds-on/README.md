# adds-on

A small Rust binary named `uv` that acts as a **plugin dispatcher** for the real `uv`. When installed before the real `uv` in `PATH`, it makes any `uv-<name>` executable callable as `uv <name>` — similar to how `kubectl` handles plugins.

## What it does

```
uv shell          →  finds uv-shell in PATH  →  exec uv-shell
uv add requests   →  no uv-add plugin found  →  exec real uv add requests
uv <TAB>          →  real uv completions + discovered plugins (dynamic)
uv shell <TAB>    →  delegates to uv-shell's own completions
```

Any executable named `uv-<name>` on PATH is automatically discovered — no registration needed.

## Setup

**1. Add to `~/.zshrc`:**
```sh
export PATH="$(brew --prefix uv-shell)/libexec/bin:$PATH"

# Optional: skip PATH scan on every uv call
export UV_REAL_PATH="/opt/homebrew/bin/uv"
```

**2. Install completions once:**
```sh
# zsh — plugins discovered dynamically at tab-press time, no re-eval needed
uv generate-shell-completion zsh > "${fpath[1]}/_uv"

# nushell — add `use ~/.config/nushell/completions/uv.nu *` to config.nu
uv generate-shell-completion nushell | save ~/.config/nushell/completions/uv.nu

# bash
echo 'eval "$(uv generate-shell-completion bash)"' >> ~/.bashrc

# fish
uv generate-shell-completion fish > ~/.config/fish/completions/uv.fish
```

## Usage

```
uv --help                              show this help
uv <plugin> [args]                     dispatch to uv-<plugin>
uv generate-shell-completion <shell>   completions with plugins injected
uv __complete                          list plugins (used by shell completions at tab-press)
```

## Environment

| Variable | Description |
|---|---|
| `UV_REAL_PATH` | Path to the real uv binary — skips PATH scan on every call |

## How it finds the real uv

Scans `PATH` for a binary named `uv` that is not itself (using `is_file()` + canonicalize comparison). Set `UV_REAL_PATH` to skip the scan entirely.

## License

[MIT](../LICENSE)
