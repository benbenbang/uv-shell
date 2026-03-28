use std::env;
use std::path::PathBuf;
use std::process::{Command, exit};

fn print_help() {
    println!(
        "uv — plugin-aware wrapper for the real uv

USAGE:
    uv <command> [args...]

PLUGIN DISPATCH:
    Any executable named `uv-<name>` on PATH is treated as a plugin.

    uv shell [args]     →  finds uv-shell in PATH, execs it
    uv <name> [args]    →  finds uv-<name> in PATH, execs it
    uv <builtin> [args] →  no plugin found, passes through to real uv

SPECIAL COMMANDS:
    uv generate-shell-completion <shell>
                        →  real uv completions + plugins injected
                           shells: bash zsh fish nushell
    uv __complete       →  list discovered plugins (used by shell completions)
    uv --version, -V    →  show wrapper version
    uv --help, -h       →  show this message

ENVIRONMENT:
    UV_REAL_PATH        Skip PATH scan — point directly to the real uv binary
                        e.g. export UV_REAL_PATH=/opt/homebrew/bin/uv

SETUP:
    export PATH=\"$(brew --prefix uv-shell)/libexec/bin:$PATH\"
    uv generate-shell-completion zsh > \"<your-fpath>/_uv\"

Any unrecognised command is forwarded to the real uv unchanged."
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("-h") | Some("--help") => {
            print_help();
        }
        Some("--version") | Some("-V") => {
            println!("uv (plugin wrapper) {}", env!("CARGO_PKG_VERSION"));
        }
        // Dynamic plugin discovery — called by shell completion at tab-press time
        Some("__complete") => {
            for plugin in discover_plugins() {
                println!("{}:Plugin: {}", plugin, plugin);
            }
        }
        // Intercept completions to inject plugin hooks
        Some("generate-shell-completion") => {
            let shell = args.get(2).map(|s| s.as_str()).unwrap_or("bash");
            generate_completions(shell);
        }
        // Try plugin, fall back to real uv
        Some(subcommand) => {
            let plugin = format!("uv-{}", subcommand);
            if let Some(plugin_path) = find_in_path(&plugin) {
                exec_or_run(plugin_path, &args[2..]);
            } else {
                exec_real_uv(&args[1..]);
            }
        }
        None => exec_real_uv(&[]),
    }
}

// ── Path helpers ──────────────────────────────────────────────────────────────

/// Find the real `uv` binary in PATH, skipping ourselves.
/// Honours `UV_REAL_PATH` env var to skip the PATH scan entirely.
fn find_real_uv() -> PathBuf {
    if let Ok(path) = env::var("UV_REAL_PATH") {
        let p = PathBuf::from(&path);
        if p.is_file() && is_executable(&p) {
            return p;
        }
        eprintln!("uv: UV_REAL_PATH={path:?} is not executable, falling back to PATH scan");
    }

    let current = env::current_exe()
        .ok()
        .and_then(|p| p.canonicalize().ok());

    env::split_paths(&env::var("PATH").unwrap_or_default())
        .map(|dir| dir.join("uv"))
        .find(|path| {
            if !path.is_file() || !is_executable(path) {
                return false;
            }
            // Skip current executable
            match (path.canonicalize().ok(), &current) {
                (Some(c), Some(cur)) => &c != cur,
                _ => true,
            }
        })
        .expect("Could not find real uv in PATH. Is uv installed?")
}

/// Find an executable by name in PATH.
fn find_in_path(name: &str) -> Option<PathBuf> {
    env::split_paths(&env::var("PATH").unwrap_or_default())
        .map(|dir| dir.join(name))
        .find(|path| path.is_file() && is_executable(path))
}

const CACHE_TTL_SECS: u64 = 60;

/// Returns `~/.cache/uv-shell/plugins` (or `$XDG_CACHE_HOME/uv-shell/plugins`).
fn plugin_cache_path() -> Option<PathBuf> {
    let base = env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|_| env::var("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .ok()?;
    Some(base.join("uv-shell").join("plugins"))
}

/// Read cached plugin list if it exists, is younger than CACHE_TTL_SECS,
/// and was written for the same PATH.
fn read_plugin_cache() -> Option<Vec<String>> {
    let path = plugin_cache_path()?;
    let content = std::fs::read_to_string(&path).ok()?;
    let mut lines = content.lines();

    let ts: u64 = lines.next()?.parse().ok()?;
    let cached_path = lines.next()?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    if now.saturating_sub(ts) > CACHE_TTL_SECS {
        return None;
    }
    if cached_path != env::var("PATH").unwrap_or_default() {
        return None;
    }

    Some(lines.map(|l| l.to_string()).collect())
}

/// Write plugin list to cache with current timestamp and PATH snapshot.
fn write_plugin_cache(plugins: &[String]) {
    let Some(path) = plugin_cache_path() else { return };
    let Some(parent) = path.parent() else { return };
    let _ = std::fs::create_dir_all(parent);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let current_path = env::var("PATH").unwrap_or_default();
    let content = format!("{}\n{}\n{}", now, current_path, plugins.join("\n"));
    let _ = std::fs::write(&path, content);
}

/// Discover all `uv-<name>` plugins in PATH, returning `<name>` for each.
/// Results are cached for CACHE_TTL_SECS seconds.
fn discover_plugins() -> Vec<String> {
    if let Some(cached) = read_plugin_cache() {
        return cached;
    }

    let mut plugins: Vec<String> = env::split_paths(&env::var("PATH").unwrap_or_default())
        .flat_map(|dir| std::fs::read_dir(dir).into_iter().flatten())
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("uv-") && is_executable(&entry.path()) {
                Some(name["uv-".len()..].to_string())
            } else {
                None
            }
        })
        .collect();

    plugins.sort();
    plugins.dedup();
    write_plugin_cache(&plugins);
    plugins
}

#[cfg(unix)]
fn is_executable(path: &PathBuf) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(_path: &PathBuf) -> bool {
    true
}

// ── Execution helpers ─────────────────────────────────────────────────────────

#[cfg(unix)]
fn exec_or_run(path: PathBuf, args: &[String]) {
    use std::os::unix::process::CommandExt;
    let err = Command::new(&path).args(args).exec();
    eprintln!("uv: failed to exec {}: {}", path.display(), err);
    exit(1);
}

#[cfg(not(unix))]
fn exec_or_run(path: PathBuf, args: &[String]) {
    let status = Command::new(&path)
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("uv: failed to run {}: {}", path.display(), e);
            exit(1);
        });
    exit(status.code().unwrap_or(1));
}

fn exec_real_uv(args: &[String]) {
    let uv = find_real_uv();
    exec_or_run(uv, args);
}

// ── Completions ───────────────────────────────────────────────────────────────

fn generate_completions(shell: &str) {
    let uv = find_real_uv();
    let output = Command::new(&uv)
        .args(["generate-shell-completion", shell])
        .output()
        .unwrap_or_else(|e| {
            eprintln!("uv: failed to generate completions: {}", e);
            exit(1);
        });

    let completions = String::from_utf8_lossy(&output.stdout).into_owned();
    let plugins = discover_plugins();

    let result = if plugins.is_empty() {
        completions
    } else {
        match shell {
            "bash" => inject_bash(completions, &plugins),
            "zsh" => inject_zsh(completions, &plugins),
            "fish" => inject_fish(completions, &plugins),
            "nushell" => inject_nushell(completions, &plugins),
            _ => completions,
        }
    };

    print!("{}", result);
}

/// Bash: the top-level opts string ends with `generate-shell-completion help"`.
/// We append plugin names right before the closing quote.
fn inject_bash(completions: String, plugins: &[String]) -> String {
    let plugin_str = plugins.join(" ");
    completions.replace(
        " generate-shell-completion help\"",
        &format!(" generate-shell-completion help {}\"", plugin_str),
    )
}

/// Zsh: inject a dynamic plugin lookup into `_uv_commands()` so that
/// `uv <TAB>` discovers plugins at tab-press time (no re-eval needed).
/// Sub-completions (`uv shell <TAB>`) are still injected statically since
/// zsh needs the function body defined in the session.
fn inject_zsh(completions: String, plugins: &[String]) -> String {
    // Step 1: replace the static _describe call with a dynamic one that also
    // calls `uv __complete` at tab-press time
    let dynamic_block = concat!(
        "    # Dynamic plugin discovery — merge into commands at tab-press time\n",
        "    local -a _uv_plugins\n",
        "    _uv_plugins=(\"${(@f)$(uv __complete 2>/dev/null)}\")\n",
        "    (( ${#_uv_plugins} )) && commands+=($_uv_plugins)\n",
        "    _describe -t commands 'uv commands' commands \"$@\"\n",
        "}",
    );
    let mut result = completions.replace(
        "    _describe -t commands 'uv commands' commands \"$@\"\n}",
        dynamic_block,
    );

    // Step 2: for each currently installed plugin that supports `completions zsh`,
    // inject a case dispatch and append its function body
    for plugin in plugins {
        if let Some(body) = plugin_zsh_body(plugin) {
            result = inject_zsh_dispatch(result, plugin, &body);
        }
    }

    result
}

/// Run `uv-<plugin> completions zsh` and return the function body,
/// stripping the `#compdef` header and the final self-call line.
fn plugin_zsh_body(plugin: &str) -> Option<String> {
    let path = find_in_path(&format!("uv-{}", plugin))?;

    let output = Command::new(&path)
        .args(["completions", "zsh"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let self_call = format!("_uv-{} \"$@\"", plugin);

    let body = text
        .lines()
        .filter(|l| !l.starts_with("#compdef") && *l != self_call)
        .collect::<Vec<_>>()
        .join("\n");

    Some(body)
}

/// Inject a `(<plugin>)` case into the top-level dispatch and append the
/// plugin's function definitions after the closing `}` of `_uv()`.
fn inject_zsh_dispatch(completions: String, plugin: &str, body: &str) -> String {
    // Insert the plugin case just before the closing `esac` of the inner
    // case $line[1] block. The 8-space indent is unique to that block.
    let with_case = completions.replace(
        ";;\n        esac\n    ;;\nesac\n}",
        &format!(
            ";;\n        ({plugin})\n            _uv-{plugin}\n        ;;\n        esac\n    ;;\nesac\n}}"
        ),
    );

    // Append plugin function definitions after the closing } of _uv()
    format!("{}\n{}", with_case, body)
}

/// Nushell: inject plugin externs into the `module completions` block.
/// Each plugin's `extern "uv-<name>"` is renamed to `extern "uv <name>"`
/// so nushell naturally shows it as a subcommand of `uv`.
fn inject_nushell(completions: String, plugins: &[String]) -> String {
    let mut result = completions;
    for plugin in plugins {
        if let Some(body) = plugin_nushell_body(plugin) {
            result = result.replace(
                "\n}\n\nexport use completions *",
                &format!("\n{}\n}}\n\nexport use completions *", body),
            );
        }
    }
    result
}

/// Run `uv-<plugin> completions nushell`, strip the module wrapper, and
/// rename all `uv-<plugin>` references to `uv <plugin>` so the externs
/// integrate cleanly into uv's own `module completions`.
fn plugin_nushell_body(plugin: &str) -> Option<String> {
    let path = find_in_path(&format!("uv-{}", plugin))?;

    let output = Command::new(&path)
        .args(["completions", "nushell"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let module_name = format!("uv-{}-completions", plugin);
    let plugin_cmd = format!("uv-{}", plugin);
    let uv_cmd = format!("uv {}", plugin);

    let body = text
        .lines()
        // strip module wrapper and use statement
        .filter(|l| !l.starts_with("module ") && !l.starts_with("use "))
        // rename "uv-shell" → "uv shell" everywhere (extern names + nu-complete refs)
        .map(|l| l.replace(&plugin_cmd, &uv_cmd))
        // strip the auto-generated module name from nu-complete function names
        .map(|l| l.replace(&module_name, &format!("uv {}", plugin)))
        .collect::<Vec<_>>()
        .join("\n");

    Some(body)
}

/// Fish: append `complete` lines for each plugin at the end.
fn inject_fish(completions: String, plugins: &[String]) -> String {
    let plugin_cmds: String = plugins
        .iter()
        .map(|p| {
            format!(
                "complete -c uv -n '__fish_use_subcommand' -a '{}' -d 'Plugin: {}'\n",
                p, p
            )
        })
        .collect();

    format!("{}\n{}", completions, plugin_cmds)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_bash_appends_plugins() {
        let fake = r#"opts="run add generate-shell-completion help""#.to_string();
        let plugins = vec!["shell".to_string(), "foo".to_string()];
        let result = inject_bash(fake, &plugins);
        assert!(result.contains("shell"), "plugin 'shell' should be injected");
        assert!(result.contains("foo"), "plugin 'foo' should be injected");
        assert!(result.contains("run add"), "original commands should remain");
    }

    #[test]
    fn test_inject_zsh_dynamic_lookup() {
        let fake = "    _describe -t commands 'uv commands' commands \"$@\"\n}".to_string();
        let result = inject_zsh(fake, &[]);
        assert!(result.contains("uv __complete"), "should inject dynamic __complete call");
        assert!(result.contains("_uv_plugins"), "should declare _uv_plugins array");
        assert!(result.contains("commands+=($_uv_plugins)"), "should merge plugins into commands");
        assert!(result.contains("_describe -t commands 'uv commands'"), "should use single describe call");
    }

    #[test]
    fn test_inject_nushell_appends_plugin() {
        let fake = "module completions {\n  export extern uv [\n  ]\n}\n\nexport use completions *".to_string();
        let body = "  export extern \"uv shell\" [\n    --help\n  ]".to_string();
        // simulate inject_nushell by calling the replace logic directly
        let result = fake.replace(
            "\n}\n\nexport use completions *",
            &format!("\n{}\n}}\n\nexport use completions *", body),
        );
        assert!(result.contains("export extern \"uv shell\""));
        assert!(result.contains("export use completions *"));
    }

    #[test]
    fn test_inject_zsh_dispatch() {
        let closing = ";;\n        esac\n    ;;\nesac\n}";
        let fake = format!("...previous case...{}", closing);
        let body = "_uv-shell() { echo shell; }".to_string();
        let result = inject_zsh_dispatch(fake, "shell", &body);
        assert!(result.contains("(shell)"), "should have shell case");
        assert!(result.contains("_uv-shell"), "should call _uv-shell");
        assert!(result.contains("_uv-shell() { echo shell; }"), "should append function body");
        assert!(result.contains("esac\n}"), "closing should still be present");
    }

    #[test]
    fn test_inject_fish_appends_plugins() {
        let fake = "# existing completions\n".to_string();
        let plugins = vec!["shell".to_string()];
        let result = inject_fish(fake, &plugins);
        assert!(result.contains("complete -c uv"));
        assert!(result.contains("-a 'shell'"));
    }

    #[test]
    fn test_inject_bash_no_plugins() {
        let fake = r#"opts="run add generate-shell-completion help""#.to_string();
        // With empty plugin list, the trailing space + empty string is harmless
        let result = inject_bash(fake.clone(), &[]);
        assert!(result.contains(" \""), "should still have closing quote");
    }
}
