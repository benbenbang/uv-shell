use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_bin_dir() -> &'static str {
    if cfg!(target_os = "windows") {
        "Scripts"
    } else {
        "bin"
    }
}

fn get_venv_path() -> PathBuf {
    let venv = PathBuf::from(".venv");
    venv.canonicalize().unwrap_or_else(|_| {
        let mut cwd = env::current_dir().expect("failed to get current directory");
        cwd.push(".venv");
        cwd
    })
}

/// Auto-detect the current shell from environment.
fn detect_shell() -> &'static str {
    if let Ok(shell) = env::var("SHELL") {
        if shell.contains("fish") {
            return "fish";
        }
        if shell.contains("nu") {
            return "nushell";
        }
        // bash, zsh, sh, etc. all use `export` syntax
        return "bash";
    }
    // No $SHELL — likely Windows
    if env::var("PSModulePath").is_ok() {
        return "powershell";
    }
    if cfg!(windows) {
        return "powershell"; // modern Windows default
    }
    "bash"
}

fn create_venv(venv_path: &PathBuf, extra_args: &[String]) {
    let mut cmd = Command::new("uv");
    cmd.arg("venv").arg(venv_path);
    for arg in extra_args {
        cmd.arg(arg);
    }

    let status = cmd.status().expect("failed to run 'uv venv'");
    if !status.success() {
        eprintln!(
            "Failed to create virtual environment at {}",
            venv_path.display()
        );
        std::process::exit(1);
    }
}

fn update_prompt(venv_path: &PathBuf, prefix: Option<&str>) {
    let cfg_path = venv_path.join("pyvenv.cfg");
    if !cfg_path.exists() {
        return;
    }

    let bin_dir = get_bin_dir();
    let python_path = venv_path.join(bin_dir).join("python");

    // Get Python version
    let output = Command::new(&python_path)
        .arg("-c")
        .arg("import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')")
        .output();

    let py_version = match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => return,
    };

    // Use --prefix if given, otherwise fall back to current directory name
    let project_name = prefix.map(|s| s.to_string()).unwrap_or_else(|| {
        env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "project".to_string())
    });

    let prompt_value = format!("{project_name}-py{py_version}");

    let content = fs::read_to_string(&cfg_path).unwrap_or_default();
    let target_line = format!("prompt = {prompt_value}");

    // Check if already correct
    if content.lines().any(|line| line == target_line) {
        return;
    }

    let has_prompt = content.lines().any(|line| line.starts_with("prompt = "));

    let new_content = if has_prompt {
        content
            .lines()
            .map(|line| {
                if line.starts_with("prompt = ") {
                    target_line.as_str()
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        format!("{}\n{target_line}", content.trim_end())
    };

    // Preserve trailing newline
    let new_content = if content.ends_with('\n') && !new_content.ends_with('\n') {
        format!("{new_content}\n")
    } else {
        new_content
    };

    let _ = fs::write(&cfg_path, new_content);
}

fn activate_venv(venv_path: &PathBuf) {
    let bin_dir = get_bin_dir();
    let venv_bin = venv_path.join(bin_dir);

    println!(
        "Activate existing virtual environment at {}",
        venv_path.display()
    );
    println!("To deactivate, hit Ctrl + D");

    // Flush before exec() replaces the process image — otherwise buffered
    // output is lost when stdout is piped.
    use std::io::Write;
    let _ = std::io::stdout().flush();

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let current_path = env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{current_path}", venv_bin.display());

        let err = Command::new(&shell)
            .env("VIRTUAL_ENV", venv_path)
            .env("PATH", new_path)
            .exec();

        eprintln!("Failed to exec shell '{shell}': {err}");
        std::process::exit(1);
    }

    #[cfg(windows)]
    {
        let path_sep = ";";
        let current_path = env::var("PATH").unwrap_or_default();
        let new_path = format!("{}{path_sep}{current_path}", venv_bin.display());

        // Detect PowerShell vs CMD
        let (shell, shell_args): (String, Vec<&str>) =
            if env::var("PSModulePath").is_ok() {
                // PowerShell — try pwsh (Core) first, fall back to powershell (Desktop)
                let exe = if Command::new("pwsh").arg("--version").output().is_ok() {
                    "pwsh".to_string()
                } else {
                    "powershell".to_string()
                };
                (exe, vec!["-NoLogo"])
            } else {
                (
                    env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string()),
                    vec![],
                )
            };

        let status = Command::new(&shell)
            .args(&shell_args)
            .env("VIRTUAL_ENV", venv_path)
            .env("PATH", new_path)
            .status()
            .expect("failed to spawn shell");

        std::process::exit(status.code().unwrap_or(1));
    }
}

fn anchor(shell_override: Option<&str>) {
    let venv_path = get_venv_path();
    let cfg_path = venv_path.join("pyvenv.cfg");

    if !venv_path.is_dir() || !cfg_path.exists() {
        return;
    }

    let bin_dir = get_bin_dir();
    let venv_bin = venv_path.join(bin_dir);

    // Touch the activated marker file
    let activated_path = venv_path.join("activated");
    if !activated_path.exists() {
        let _ = fs::File::create(&activated_path);
    }

    let shell = shell_override.unwrap_or_else(|| detect_shell());
    let venv = venv_path.display();
    let bin = venv_bin.display();

    match shell {
        "fish" => {
            println!("set -gx VIRTUAL_ENV \"{venv}\"");
            println!("set -gx PIP_REQUIRE_VIRTUALENV false");
            println!("set -gp PATH \"{bin}\"");
        }
        "nushell" => {
            println!("$env.VIRTUAL_ENV = \"{venv}\"");
            println!("$env.PIP_REQUIRE_VIRTUALENV = \"false\"");
            println!("$env.PATH = ($env.PATH | prepend \"{bin}\")");
        }
        "powershell" => {
            println!("$env:VIRTUAL_ENV = \"{venv}\"");
            println!("$env:PIP_REQUIRE_VIRTUALENV = \"false\"");
            println!("$env:PATH = \"{bin}\" + [IO.Path]::PathSeparator + $env:PATH");
        }
        "cmd" => {
            println!("set \"VIRTUAL_ENV={venv}\"");
            println!("set \"PIP_REQUIRE_VIRTUALENV=false\"");
            println!("set \"PATH={bin};%PATH%\"");
        }
        _ => {
            // bash, zsh, sh, etc.
            println!("export VIRTUAL_ENV=\"{venv}\"");
            println!("export PIP_REQUIRE_VIRTUALENV=false");
            println!("export PATH=\"{bin}:$PATH\"");
        }
    }
}

fn print_help() {
    println!(
        "\
uv-shell — create and activate virtual environments with uv

Usage: uv-shell [OPTIONS]
       uv-shell anchor [--shell <SHELL>]
       uv-shell completions <SHELL>

Commands:
  anchor           Print export commands for shell rc eval (auto-detects shell)
                     bash/zsh : eval \"$(uv-shell anchor)\"
                     fish     : uv-shell anchor | source
                     nushell  : uv-shell anchor --shell nushell | save -f ~/.venv-anchor.nu
                     powershell: uv-shell anchor | Invoke-Expression
  completions      Generate shell completions (bash, zsh, fish, nushell, powershell)

  (default)        Create .venv if missing, then spawn an activated subshell

Anchor options:
      --shell <SHELL>         Override shell detection (bash, fish, nushell, powershell, cmd)

Options (forwarded to `uv venv`):
  -p, --python <PYTHON>         Python interpreter to use
      --seed                    Install seed packages (pip, setuptools, wheel)
  -c, --clear                   Re-create the venv even if .venv already exists
      --allow-existing          Preserve existing files at the target path
      --prefix <PREFIX>         Override project name in prompt (keeps -pyX.XX suffix)
      --prompt <PROMPT>         Full custom prompt (replaces auto-generated name entirely)
      --system-site-packages    Access system site packages
      --relocatable             Make the venv relocatable
      --no-project              Avoid discovering a project or workspace
      --no-config               Avoid discovering configuration files
  -q, --quiet                   Quiet output
  -v, --verbose                 Verbose output
  -h, --help                    Show this help message

All other `uv venv` options are also forwarded. See `uv venv --help` for the full list."
    );
}

fn print_completions(shell: &str) {
    match shell {
        "bash" => print!("{}", include_str!("completions/uv-shell.bash")),
        "zsh" => print!("{}", include_str!("completions/uv-shell.zsh")),
        "fish" => print!("{}", include_str!("completions/uv-shell.fish")),
        "nushell" => print!("{}", include_str!("completions/uv-shell.nu")),
        "powershell" => print!("{}", include_str!("completions/uv-shell.ps1")),
        _ => {
            eprintln!("Unsupported shell: {shell}");
            eprintln!("Supported shells: bash, zsh, fish, nushell, powershell");
            std::process::exit(1);
        }
    }
}

/// Scan args for flags we need to know about, without consuming them.
/// Returns (has_clear, has_custom_prompt, has_help, prefix).
fn scan_flags(args: &[String]) -> (bool, bool, bool, Option<String>) {
    let mut has_clear = false;
    let mut has_prompt = false;
    let mut has_help = false;
    let mut prefix: Option<String> = None;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-c" | "--clear" => has_clear = true,
            "--prompt" => {
                has_prompt = true;
                iter.next(); // skip the value
            }
            "--prefix" => {
                prefix = iter.next().map(|s| s.to_string());
            }
            "-h" | "--help" => has_help = true,
            _ => {
                if arg.starts_with("--prompt=") {
                    has_prompt = true;
                } else if let Some(val) = arg.strip_prefix("--prefix=") {
                    prefix = Some(val.to_string());
                }
            }
        }
    }

    (has_clear, has_prompt, has_help, prefix)
}

/// Parse `anchor` subcommand args for `--shell <name>`.
fn parse_anchor_shell<'a>(args: &'a [String]) -> Option<&'a str> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--shell" {
            return iter.next().map(|s| s.as_str());
        }
        if let Some(val) = arg.strip_prefix("--shell=") {
            return Some(val);
        }
    }
    None
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // args[0] = binary, args[1..] = user args
    let user_args = &args[1..];

    // Subcommand dispatch
    if let Some(first) = user_args.first() {
        match first.as_str() {
            "anchor" => {
                let anchor_args = &user_args[1..];
                let shell_override = parse_anchor_shell(anchor_args);
                anchor(shell_override);
                return;
            }
            "completions" => {
                match user_args.get(1).map(|s| s.as_str()) {
                    Some(shell) => print_completions(shell),
                    None => {
                        eprintln!(
                            "Usage: uv-shell completions <bash|zsh|fish|nushell|powershell>"
                        );
                        std::process::exit(1);
                    }
                }
                return;
            }
            _ => {}
        }
    }

    let (has_clear, has_custom_prompt, has_help, prefix) = scan_flags(user_args);

    if has_help {
        print_help();
        return;
    }

    let venv_path = get_venv_path();
    let needs_create = !venv_path.is_dir() || has_clear;

    if needs_create {
        // Strip --prefix and its value — it's our flag, unknown to `uv venv`
        let forwarded: Vec<String> = {
            let mut out = Vec::new();
            let mut iter = user_args.iter();
            while let Some(arg) = iter.next() {
                if arg == "--prefix" {
                    iter.next(); // skip value
                } else if arg.starts_with("--prefix=") {
                    // skip entirely
                } else {
                    out.push(arg.clone());
                }
            }
            out
        };
        create_venv(&venv_path, &forwarded);
    }

    if !has_custom_prompt {
        update_prompt(&venv_path, prefix.as_deref());
    }

    if venv_path.is_dir() {
        activate_venv(&venv_path);
    } else {
        eprintln!(
            "Failed to create virtual environment at {}",
            venv_path.display()
        );
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &[&str]) -> Vec<String> {
        s.iter().map(|a| a.to_string()).collect()
    }

    // ── scan_flags ──────────────────────────────────────────────

    #[test]
    fn scan_flags_empty() {
        let (clear, prompt, help, _) = scan_flags(&args(&[]));
        assert!(!clear);
        assert!(!prompt);
        assert!(!help);
    }

    #[test]
    fn scan_flags_clear_short() {
        let (clear, _, _, _) = scan_flags(&args(&["-c"]));
        assert!(clear);
    }

    #[test]
    fn scan_flags_clear_long() {
        let (clear, _, _, _) = scan_flags(&args(&["--clear"]));
        assert!(clear);
    }

    #[test]
    fn scan_flags_prompt_space() {
        let (_, prompt, _, _) = scan_flags(&args(&["--prompt", "my-env"]));
        assert!(prompt);
    }

    #[test]
    fn scan_flags_prompt_eq() {
        let (_, prompt, _, _) = scan_flags(&args(&["--prompt=my-env"]));
        assert!(prompt);
    }

    #[test]
    fn scan_flags_help_short() {
        let (_, _, help, _) = scan_flags(&args(&["-h"]));
        assert!(help);
    }

    #[test]
    fn scan_flags_help_long() {
        let (_, _, help, _) = scan_flags(&args(&["--help"]));
        assert!(help);
    }

    #[test]
    fn scan_flags_combined() {
        let (clear, prompt, help, _) =
            scan_flags(&args(&["-p", "3.12", "--clear", "--prompt", "x", "-v"]));
        assert!(clear);
        assert!(prompt);
        assert!(!help);
    }

    #[test]
    fn scan_flags_prefix_space() {
        let (_, _, _, prefix) = scan_flags(&args(&["--prefix", "myproject"]));
        assert_eq!(prefix.as_deref(), Some("myproject"));
    }

    #[test]
    fn scan_flags_prefix_eq() {
        let (_, _, _, prefix) = scan_flags(&args(&["--prefix=myproject"]));
        assert_eq!(prefix.as_deref(), Some("myproject"));
    }

    #[test]
    fn scan_flags_prompt_value_not_misread() {
        // The value after --prompt should be skipped, not treated as a flag
        let (clear, prompt, help, _) = scan_flags(&args(&["--prompt", "--clear"]));
        assert!(prompt);
        // "--clear" is consumed as the prompt value, not as the clear flag
        assert!(!clear);
        assert!(!help);
    }

    // ── get_bin_dir ─────────────────────────────────────────────

    #[test]
    fn bin_dir_is_correct() {
        let d = get_bin_dir();
        if cfg!(target_os = "windows") {
            assert_eq!(d, "Scripts");
        } else {
            assert_eq!(d, "bin");
        }
    }

    // ── parse_anchor_shell ──────────────────────────────────────

    #[test]
    fn anchor_shell_none() {
        assert_eq!(parse_anchor_shell(&args(&[])), None);
    }

    #[test]
    fn anchor_shell_space() {
        assert_eq!(
            parse_anchor_shell(&args(&["--shell", "fish"])),
            Some("fish")
        );
    }

    #[test]
    fn anchor_shell_eq() {
        assert_eq!(
            parse_anchor_shell(&args(&["--shell=powershell"])),
            Some("powershell")
        );
    }

    // ── detect_shell ────────────────────────────────────────────

    #[test]
    fn detect_shell_returns_known_value() {
        let s = detect_shell();
        assert!(
            ["bash", "fish", "nushell", "powershell"].contains(&s),
            "unexpected shell: {s}"
        );
    }
}
