//! Smoke test CLI untuk biner `homeostasis-node` (MOD-11).

#![forbid(unsafe_code)]

use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_homeostasis-node");

#[test]
fn version_flag_prints_package_version() {
    let out = Command::new(BIN).arg("--version").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        stdout,
        format!("homeostasis-node {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn help_and_no_args_print_placeholder_info() {
    for args in [&[][..], &["--help"][..]] {
        let out = Command::new(BIN).args(args).output().unwrap();
        assert_eq!(out.status.code(), Some(0));
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert!(stdout.contains("homeostasis-node"));
        assert!(stdout.contains("MOD-11"));
        assert!(stdout.contains("kerangka R0, belum berfungsi"));
    }
}

#[test]
fn unknown_arg_exits_with_code_two() {
    let out = Command::new(BIN).arg("--tidak-dikenal").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(!out.stderr.is_empty());
}
