use assert_cmd::prelude::*;
use predicates::str::contains;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use tempfile::{NamedTempFile, TempPath};

fn make_hook_script() -> TempPath {
    let mut file = NamedTempFile::new().unwrap();
    fs::write(
        file.path(),
        "#!/bin/sh\ncat >/dev/null\nprintf '{\"decision\":\"approve\",\"reason\":\"ok\"}'\n",
    )
    .unwrap();
    fs::set_permissions(file.path(), fs::Permissions::from_mode(0o755)).unwrap();
    file.into_temp_path()
}

#[test]
fn test_help() {
    Command::cargo_bin("hooktest")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("hooktest"));
}

#[test]
fn test_pretool() {
    let hook = make_hook_script();
    Command::cargo_bin("hooktest")
        .unwrap()
        .args([
            "pretool",
            "--tool",
            "Bash",
            "--",
            hook.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("Decision: APPROVE"));
}

#[test]
fn test_posttool() {
    let hook = make_hook_script();
    Command::cargo_bin("hooktest")
        .unwrap()
        .args([
            "posttool",
            "--tool",
            "Bash",
            "--tool-response",
            "output=ok",
            "--",
            hook.to_path_buf().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("Hook Output (Parsed)"));
}

#[test]
fn test_notification() {
    Command::cargo_bin("hooktest")
        .unwrap()
        .args(["notification", "--", "true"])
        .assert()
        .success()
        .stdout(contains("Running Hook"));
}

#[test]
fn test_stop() {
    let hook = make_hook_script();
    Command::cargo_bin("hooktest")
        .unwrap()
        .args(["stop", "--", hook.to_path_buf().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("Hook Output (Parsed)"));
}

#[test]
fn test_subagent_stop() {
    let hook = make_hook_script();
    Command::cargo_bin("hooktest")
        .unwrap()
        .args(["subagentstop", "--", hook.to_path_buf().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("Hook Output (Parsed)"));
}
