use std::env;
use std::path::PathBuf;
use std::process::{Command, exit};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        // Intercept completions to inject discovered plugins
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

/// Discover all `uv-<name>` plugins in PATH, returning `<name>` for each.
fn discover_plugins() -> Vec<String> {
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

/// Zsh: inject plugins into `_uv_commands()` list, add case dispatch, and
/// append each plugin's completion function body.
fn inject_zsh(completions: String, plugins: &[String]) -> String {
    // Step 1: add plugins to the _uv_commands() list
    let plugin_entries: String = plugins
        .iter()
        .map(|p| format!("'{}:Plugin: {}' \\\n", p, p))
        .collect();

    let mut result = completions.replace(
        "'help:Display documentation for a command' \\\n    )",
        &format!(
            "'help:Display documentation for a command' \\\n{}    )",
            plugin_entries
        ),
    );

    // Step 2: for each plugin that supports `completions zsh`, inject a case
    // dispatch and append its function definitions
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
    fn test_inject_zsh_appends_plugins() {
        let fake = "'help:Display documentation for a command' \\\n    )".to_string();
        let plugins = vec!["shell".to_string()];
        let result = inject_zsh(fake, &plugins);
        assert!(result.contains("'shell:Plugin: shell'"));
        assert!(result.contains("'help:Display documentation for a command'"));
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
