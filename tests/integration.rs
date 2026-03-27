//! Integration tests for uv-shell.
//!
//! These require `uv` to be installed and on PATH.
//! Tests that invoke default mode (venv creation + activation) set
//! SHELL=/usr/bin/true so the exec'd process exits immediately.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_uv-shell"))
}

/// Create a fresh temp directory for a single test.
fn tmpdir(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("uv-shell-test-{name}"));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

/// Create a venv in a temp dir, return the dir.
fn tmpdir_with_venv(name: &str) -> PathBuf {
    let dir = tmpdir(name);
    assert!(Command::new("uv")
        .args(["venv", ".venv"])
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());
    dir
}

// ── help ────────────────────────────────────────────────────────

#[test]
fn help_long_flag() {
    let out = Command::new(bin()).arg("--help").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("uv-shell"));
    assert!(stdout.contains("anchor"));
    assert!(stdout.contains("--python"));
    assert!(stdout.contains("--clear"));
    assert!(stdout.contains("--prompt"));
    assert!(stdout.contains("--shell"));
    assert!(stdout.contains("powershell"));
}

#[test]
fn help_short_flag() {
    let out = Command::new(bin()).arg("-h").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Usage:"));
}

// ── anchor (default / bash) ─────────────────────────────────────

#[test]
fn anchor_no_venv_reports_on_stderr() {
    let dir = tmpdir("anchor-none");
    let out = Command::new(bin())
        .arg("anchor")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        out.stdout.is_empty(),
        "anchor should produce no stdout without .venv"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no .venv found"),
        "anchor should explain why it produced no output, got: {stderr}"
    );
    assert!(
        stderr.contains("or any parent directory"),
        "anchor should mention traversal in message, got: {stderr}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn anchor_prints_exports_bash() {
    let dir = tmpdir_with_venv("anchor-bash");

    let out = Command::new(bin())
        .args(["anchor", "--shell", "bash"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("export VIRTUAL_ENV=\""));
    assert!(stdout.contains("export PIP_REQUIRE_VIRTUALENV=false"));
    assert!(stdout.contains("export PATH=\""));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn anchor_finds_venv_in_parent_directory() {
    // project root has .venv; anchor is run from a subdirectory
    let dir = tmpdir_with_venv("anchor-traversal");
    let subdir = dir.join("src/utils");
    fs::create_dir_all(&subdir).unwrap();

    let out = Command::new(bin())
        .args(["anchor", "--shell", "bash"])
        .current_dir(&subdir)
        .output()
        .unwrap();
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("export VIRTUAL_ENV=\""),
        "anchor should find .venv two levels up, got:\n{stdout}"
    );
    // The VIRTUAL_ENV path should point to the root .venv, not a subdir .venv
    assert!(
        stdout.contains(dir.join(".venv").to_str().unwrap()),
        "VIRTUAL_ENV should point to root .venv"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn anchor_creates_activated_marker() {
    let dir = tmpdir_with_venv("anchor-marker");

    assert!(!dir.join(".venv/activated").exists());

    Command::new(bin())
        .arg("anchor")
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(dir.join(".venv/activated").exists());
    let _ = fs::remove_dir_all(&dir);
}

// ── anchor --shell (per-shell output formats) ───────────────────

#[test]
fn anchor_shell_fish() {
    let dir = tmpdir_with_venv("anchor-fish");

    let out = Command::new(bin())
        .args(["anchor", "--shell", "fish"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("set -gx VIRTUAL_ENV"),
        "fish anchor should use set -gx, got:\n{stdout}"
    );
    assert!(stdout.contains("set -gx PIP_REQUIRE_VIRTUALENV false"));
    assert!(stdout.contains("set -gp PATH"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn anchor_shell_powershell() {
    let dir = tmpdir_with_venv("anchor-ps");

    let out = Command::new(bin())
        .args(["anchor", "--shell", "powershell"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("$env:VIRTUAL_ENV"),
        "powershell anchor should use $env:, got:\n{stdout}"
    );
    assert!(stdout.contains("$env:PIP_REQUIRE_VIRTUALENV"));
    assert!(stdout.contains("$env:PATH"));
    assert!(stdout.contains("[IO.Path]::PathSeparator"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn anchor_shell_nushell() {
    let dir = tmpdir_with_venv("anchor-nu");

    let out = Command::new(bin())
        .args(["anchor", "--shell", "nushell"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("$env.VIRTUAL_ENV"),
        "nushell anchor should use $env., got:\n{stdout}"
    );
    assert!(stdout.contains("$env.PIP_REQUIRE_VIRTUALENV"));
    assert!(stdout.contains("$env.PATH = ($env.PATH | prepend"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn anchor_shell_cmd() {
    let dir = tmpdir_with_venv("anchor-cmd");

    let out = Command::new(bin())
        .args(["anchor", "--shell", "cmd"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("set \"VIRTUAL_ENV="),
        "cmd anchor should use set, got:\n{stdout}"
    );
    assert!(stdout.contains("set \"PIP_REQUIRE_VIRTUALENV=false\""));
    assert!(stdout.contains("set \"PATH="));
    assert!(stdout.contains(";%PATH%\""));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn anchor_shell_eq_syntax() {
    let dir = tmpdir_with_venv("anchor-eq");

    let out = Command::new(bin())
        .args(["anchor", "--shell=fish"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("set -gx VIRTUAL_ENV"));
    let _ = fs::remove_dir_all(&dir);
}

// ── default mode (create + activate) ────────────────────────────

#[test]
fn creates_venv_when_missing() {
    let dir = tmpdir("create-new");
    assert!(!dir.join(".venv").exists());

    let out = Command::new(bin())
        .env("SHELL", "/usr/bin/true")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());

    // Venv created
    assert!(dir.join(".venv").is_dir());
    assert!(dir.join(".venv/pyvenv.cfg").exists());

    // Stdout shows activation message
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Activate existing virtual environment"));
    assert!(stdout.contains("Ctrl + D"));

    // Auto-prompt was set
    let cfg = fs::read_to_string(dir.join(".venv/pyvenv.cfg")).unwrap();
    assert!(
        cfg.contains("prompt = uv-shell-test-create-new-py"),
        "expected auto-prompt in pyvenv.cfg, got:\n{cfg}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn existing_venv_not_recreated() {
    let dir = tmpdir_with_venv("keep-existing");

    // Drop a marker file
    fs::write(dir.join(".venv/marker"), "keep").unwrap();

    let out = Command::new(bin())
        .env("SHELL", "/usr/bin/true")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());

    // Marker still present -> venv was not wiped
    assert_eq!(
        fs::read_to_string(dir.join(".venv/marker")).unwrap(),
        "keep"
    );
    let _ = fs::remove_dir_all(&dir);
}

// ── --clear ─────────────────────────────────────────────────────

#[test]
fn clear_recreates_venv() {
    let dir = tmpdir_with_venv("clear-recreate");

    fs::write(dir.join(".venv/marker"), "old").unwrap();

    let out = Command::new(bin())
        .arg("--clear")
        .env("SHELL", "/usr/bin/true")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());

    // Venv still exists but marker is gone -> recreated
    assert!(dir.join(".venv").is_dir());
    assert!(!dir.join(".venv/marker").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn clear_short_flag() {
    let dir = tmpdir_with_venv("clear-short");

    fs::write(dir.join(".venv/marker"), "old").unwrap();

    let out = Command::new(bin())
        .arg("-c")
        .env("SHELL", "/usr/bin/true")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(!dir.join(".venv/marker").exists());
    let _ = fs::remove_dir_all(&dir);
}

// ── --prompt ────────────────────────────────────────────────────

#[test]
fn custom_prompt_respected() {
    let dir = tmpdir("custom-prompt");

    let out = Command::new(bin())
        .args(["--prompt", "my-fancy-env"])
        .env("SHELL", "/usr/bin/true")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());

    let cfg = fs::read_to_string(dir.join(".venv/pyvenv.cfg")).unwrap();
    assert!(
        cfg.contains("prompt = my-fancy-env"),
        "expected custom prompt, got:\n{cfg}"
    );
    // Auto-prompt should NOT have overwritten it
    assert!(
        !cfg.contains("uv-shell-test-custom-prompt-py"),
        "auto-prompt should not appear when --prompt is given"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn prompt_eq_syntax() {
    let dir = tmpdir("prompt-eq");

    let out = Command::new(bin())
        .arg("--prompt=eq-env")
        .env("SHELL", "/usr/bin/true")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());

    let cfg = fs::read_to_string(dir.join(".venv/pyvenv.cfg")).unwrap();
    assert!(cfg.contains("prompt = eq-env"));
    let _ = fs::remove_dir_all(&dir);
}

// ── --python / -p ───────────────────────────────────────────────

#[test]
fn python_version_forwarded() {
    let dir = tmpdir("py-version");

    let out = Command::new(bin())
        .args(["-p", "3.12"])
        .env("SHELL", "/usr/bin/true")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());

    let cfg = fs::read_to_string(dir.join(".venv/pyvenv.cfg")).unwrap();
    assert!(
        cfg.contains("version_info = 3.12"),
        "expected Python 3.12 in cfg, got:\n{cfg}"
    );
    assert!(cfg.contains("prompt = uv-shell-test-py-version-py3.12"));
    let _ = fs::remove_dir_all(&dir);
}

// ── completions ─────────────────────────────────────────────────

#[test]
fn completions_bash() {
    let out = Command::new(bin())
        .args(["completions", "bash"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("complete -F _uv_shell uv-shell"));
    assert!(stdout.contains("anchor"));
    assert!(stdout.contains("completions"));
}

#[test]
fn completions_zsh() {
    let out = Command::new(bin())
        .args(["completions", "zsh"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("#compdef uv-shell"));
    assert!(stdout.contains("anchor"));
    assert!(stdout.contains("completions"));
}

#[test]
fn completions_fish() {
    let out = Command::new(bin())
        .args(["completions", "fish"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("complete -c uv-shell"));
    assert!(stdout.contains("anchor"));
    assert!(stdout.contains("completions"));
}

#[test]
fn completions_nushell() {
    let out = Command::new(bin())
        .args(["completions", "nushell"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("module uv-shell-completions"));
    assert!(stdout.contains("extern \"uv-shell\""));
    assert!(stdout.contains("extern \"uv-shell anchor\""));
    assert!(stdout.contains("extern \"uv-shell completions\""));
}

#[test]
fn completions_powershell() {
    let out = Command::new(bin())
        .args(["completions", "powershell"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Register-ArgumentCompleter"));
    assert!(stdout.contains("uv-shell"));
    assert!(stdout.contains("anchor"));
    assert!(stdout.contains("completions"));
}

#[test]
fn completions_unsupported_shell() {
    let out = Command::new(bin())
        .args(["completions", "tcsh"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Unsupported shell: tcsh"));
}

#[test]
fn completions_missing_arg() {
    let out = Command::new(bin())
        .arg("completions")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Usage:"));
}

// ── --seed ──────────────────────────────────────────────────────

#[test]
fn seed_installs_pip() {
    let dir = tmpdir("seed-flag");

    let out = Command::new(bin())
        .arg("--seed")
        .env("SHELL", "/usr/bin/true")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());

    // With --seed, pip should be installed in the venv
    let pip = dir.join(".venv/bin/pip");
    assert!(pip.exists(), "pip should be installed with --seed");
    let _ = fs::remove_dir_all(&dir);
}
