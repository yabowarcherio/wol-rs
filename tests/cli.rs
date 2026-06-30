//! Black-box tests for the `wol-rs` binary.
//!
//! These exercise the binary through `--dry-run`, which prints the packet to
//! stdout instead of opening a socket — that way the test suite never touches
//! the network and runs on any CI image.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wol-rs"))
}

fn stdout(mut cmd: Command) -> (i32, String, String) {
    let out = cmd.output().expect("failed to spawn wol-rs");
    let code = out.status.code().unwrap_or(-1);
    let so = String::from_utf8(out.stdout).expect("stdout not utf-8");
    let se = String::from_utf8(out.stderr).expect("stderr not utf-8");
    (code, so, se)
}

#[test]
fn dry_run_prints_102_byte_packet() {
    let (code, out, _) = stdout({
        let mut c = bin();
        c.args(["--dry-run", "aa:bb:cc:dd:ee:ff"]);
        c
    });
    assert_eq!(code, 0);
    let line = out.trim();
    assert_eq!(line.len(), 204, "102 bytes => 204 hex chars, got {line}");
    assert!(line.starts_with("FFFFFFFFFFFF"), "header missing: {line}");
    // First MAC repetition right after the 12-char header.
    assert_eq!(&line[12..24], "AABBCCDDEEFF");
}

#[test]
fn dry_run_with_password_prints_108_byte_packet() {
    let (code, out, _) = stdout({
        let mut c = bin();
        c.args([
            "--dry-run",
            "--password",
            "DE:AD:BE:EF:DE:AD",
            "00:11:22:33:44:55",
        ]);
        c
    });
    assert_eq!(code, 0);
    let line = out.trim();
    assert_eq!(line.len(), 216, "108 bytes => 216 hex chars, got {line}");
    // Last six bytes are the SecureOn password.
    assert!(line.ends_with("DEADBEEFDEAD"));
}

#[test]
fn dry_run_emits_one_line_per_mac() {
    let (code, out, _) = stdout({
        let mut c = bin();
        c.args(["--dry-run", "aa:bb:cc:dd:ee:ff", "11:22:33:44:55:66"]);
        c
    });
    assert_eq!(code, 0);
    let lines: Vec<_> = out.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("AABBCCDDEEFF"));
    assert!(lines[1].contains("112233445566"));
}

#[test]
fn dry_run_accepts_mixed_mac_spellings() {
    let (code, out, _) = stdout({
        let mut c = bin();
        c.args([
            "--dry-run",
            "aa:bb:cc:dd:ee:ff",
            "aa-bb-cc-dd-ee-ff",
            "aabb.ccdd.eeff",
            "aabbccddeeff",
        ]);
        c
    });
    assert_eq!(code, 0);
    let lines: Vec<_> = out.lines().collect();
    assert_eq!(lines.len(), 4);
    // Every spelling resolves to the same canonical packet.
    let first = lines[0];
    for l in &lines[1..] {
        assert_eq!(*l, first, "spelling produced a different packet");
    }
}

#[test]
fn no_macs_exits_two() {
    let (code, _, err) = stdout(bin());
    assert_eq!(code, 2, "clap should exit with 2 on missing args");
    assert!(err.contains("required") || err.contains("MAC"));
}

#[test]
fn bad_mac_exits_two() {
    let (code, _, err) = stdout({
        let mut c = bin();
        c.args(["--dry-run", "not-a-mac!"]);
        c
    });
    assert_eq!(code, 2);
    assert!(err.contains("wol-rs:"), "error prefix missing: {err}");
}

#[test]
fn bad_password_exits_two() {
    let (code, _, err) = stdout({
        let mut c = bin();
        c.args(["--dry-run", "--password", "nope", "aa:bb:cc:dd:ee:ff"]);
        c
    });
    assert_eq!(code, 2);
    assert!(err.contains("password"), "error mentioning password: {err}");
}

#[test]
fn dry_run_conflicts_with_broadcast() {
    let (code, _, err) = stdout({
        let mut c = bin();
        c.args([
            "--dry-run",
            "--broadcast",
            "192.168.1.255",
            "aa:bb:cc:dd:ee:ff",
        ]);
        c
    });
    assert_eq!(code, 2);
    assert!(err.contains("cannot be used with"));
}

#[test]
fn repeat_zero_is_rejected() {
    let (code, _, err) = stdout({
        let mut c = bin();
        c.args(["--dry-run", "--repeat", "0", "aa:bb:cc:dd:ee:ff"]);
        c
    });
    assert_eq!(code, 2);
    assert!(
        err.contains("--repeat"),
        "stderr should mention --repeat: {err}"
    );
}

#[test]
fn repeat_with_dry_run_prints_once() {
    // The dry-run path emits the bytes once even when --repeat > 1, because
    // the bytes don't change between sends and we don't want to spam stdout.
    let (code, out, _) = stdout({
        let mut c = bin();
        c.args(["--dry-run", "--repeat", "5", "aa:bb:cc:dd:ee:ff"]);
        c
    });
    assert_eq!(code, 0);
    let lines: Vec<_> = out.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 1, "expected one line, got {lines:?}");
}

#[test]
fn help_and_version_succeed() {
    for flag in ["--help", "-h", "--version", "-V"] {
        let (code, out, _) = stdout({
            let mut c = bin();
            c.arg(flag);
            c
        });
        assert_eq!(code, 0, "{flag} should succeed");
        assert!(!out.is_empty(), "{flag} should print something");
    }
}
