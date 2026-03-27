//! Integration tests for the uv plugin wrapper (adds-on).
//!
//! Tests run the compiled wrapper binary directly.  UV_REAL_PATH is always set
//! to the real uv so tests are independent of the caller's PATH ordering.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

const REAL_UV: &str = "/opt/homebrew/bin/uv";

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_uv"))
}

fn wrapper(args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .env("UV_REAL_PATH", REAL_UV)
        .output()
        .unwrap()
}

// ── help ────────────────────────────────────────────────────────

#[test]
fn help_long_flag() {
    let out = wrapper(&["--help"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("plugin-aware wrapper"));
    assert!(stdout.contains("generate-shell-completion"));
    assert!(stdout.contains("UV_REAL_PATH"));
    assert!(stdout.contains("__complete"));
}

#[test]
fn help_short_flag() {
    let out = wrapper(&["-h"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("plugin-aware wrapper"));
}

// ── version ─────────────────────────────────────────────────────

#[test]
fn version_long_flag() {
    let out = wrapper(&["--version"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("uv (plugin wrapper)"),
        "expected version line, got: {stdout}"
    );
}

#[test]
fn version_short_flag() {
    let out = wrapper(&["-V"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("uv (plugin wrapper)"));
}

// ── __complete ──────────────────────────────────────────────────

/// __complete with an empty PATH returns no output.
#[test]
fn complete_empty_path() {
    let out = Command::new(bin())
        .arg("__complete")
        .env("UV_REAL_PATH", REAL_UV)
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(out.stdout.is_empty(), "expected no plugins, got output");
}

/// __complete discovers a fake uv-* executable placed on PATH.
#[test]
fn complete_discovers_fake_plugin() {
    let dir = make_fake_plugin("testplugin");

    let out = Command::new(bin())
        .arg("__complete")
        .env("UV_REAL_PATH", REAL_UV)
        .env("PATH", dir.path())
        .output()
        .unwrap();
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("testplugin"),
        "expected 'testplugin' in __complete output, got:\n{stdout}"
    );
}

// ── generate-shell-completion ────────────────────────────────────

#[test]
fn completions_bash() {
    let out = wrapper(&["generate-shell-completion", "bash"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("generate-shell-completion"));
}

#[test]
fn completions_zsh() {
    let out = wrapper(&["generate-shell-completion", "zsh"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // uv's zsh completions always define _uv and _uv_commands
    assert!(stdout.contains("_uv"));
    assert!(stdout.contains("_uv_commands"));
}

#[test]
fn completions_fish() {
    let out = wrapper(&["generate-shell-completion", "fish"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("complete -c uv"));
}

#[test]
fn completions_nushell() {
    let out = wrapper(&["generate-shell-completion", "nushell"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("module completions"));
}

/// Plugins discovered at completion-generation time are injected into zsh output.
#[test]
fn completions_zsh_injects_plugin() {
    let dir = make_fake_plugin("myplugin");

    // The fake plugin doesn't implement `completions zsh`, so only the
    // dynamic __complete hook is relevant here — we just check the
    // dynamic lookup block is present (always injected).
    let out = Command::new(bin())
        .args(["generate-shell-completion", "zsh"])
        .env("UV_REAL_PATH", REAL_UV)
        .env("PATH", format!("{}:{REAL_UV}/../..", dir.path().display()))
        .output()
        .unwrap();
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("uv __complete"),
        "expected dynamic __complete hook in zsh output"
    );
}

// ── plugin dispatch ──────────────────────────────────────────────

/// When a uv-<name> binary is on PATH, `uv <name>` execs it.
#[test]
fn plugin_dispatch_executes_plugin() {
    // Create a fake plugin that prints a unique marker and exits 0
    let tmp = tempdir();
    let plugin = tmp.join("uv-greet");
    fs::write(
        &plugin,
        "#!/bin/sh\necho 'hello-from-uv-greet'\n",
    )
    .unwrap();
    fs::set_permissions(&plugin, fs::Permissions::from_mode(0o755)).unwrap();

    let out = Command::new(bin())
        .arg("greet")
        .env("UV_REAL_PATH", REAL_UV)
        .env("PATH", &tmp)
        .output()
        .unwrap();
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("hello-from-uv-greet"),
        "expected plugin output, got:\n{stdout}"
    );
}

/// Arguments after the subcommand are forwarded to the plugin.
#[test]
fn plugin_dispatch_forwards_args() {
    let tmp = tempdir();
    let plugin = tmp.join("uv-echo");
    fs::write(&plugin, "#!/bin/sh\necho \"args: $*\"\n").unwrap();
    fs::set_permissions(&plugin, fs::Permissions::from_mode(0o755)).unwrap();

    let out = Command::new(bin())
        .args(["echo", "--foo", "bar"])
        .env("UV_REAL_PATH", REAL_UV)
        .env("PATH", &tmp)
        .output()
        .unwrap();
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("--foo") && stdout.contains("bar"),
        "args not forwarded: {stdout}"
    );
}

/// Unknown subcommand with no matching plugin passes through to real uv.
#[test]
fn passthrough_to_real_uv() {
    // `uv self version` is a real uv subcommand that works without a project
    let out = wrapper(&["self", "version"]);
    assert!(
        out.status.success(),
        "passthrough to real uv failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("uv"),
        "expected uv version output, got:\n{stdout}"
    );
}

// ── helpers ──────────────────────────────────────────────────────

fn tempdir() -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "uv-addon-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    ));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

struct TempPlugin {
    _dir: PathBuf,
}

impl TempPlugin {
    fn path(&self) -> &PathBuf {
        &self._dir
    }
}

/// Create a temp dir with a `uv-<name>` executable that exits 0.
fn make_fake_plugin(name: &str) -> TempPlugin {
    let dir = tempdir();
    let bin = dir.join(format!("uv-{name}"));
    fs::write(&bin, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
    TempPlugin { _dir: dir }
}
