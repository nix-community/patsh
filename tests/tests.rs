use assert_cmd::Command;
use expect_test::expect_file;
use tempfile::tempdir;

use std::{fs, str};

#[test]
fn basic() {
    check("basic");
}

#[test]
fn escape() {
    check("escape");
}

fn check(name: &str) {
    let dir = tempdir().unwrap();
    let out = dir.path().join(format!("{name}-actual.sh"));

    Command::new(env!("CARGO_BIN_EXE_patsh"))
        .arg(format!("tests/fixtures/{name}.sh"))
        .arg(&out)
        .unwrap();

    expect_file!(format!("fixtures/{name}-expected.sh"))
        .assert_eq(str::from_utf8(&fs::read(out).unwrap()).unwrap())
}
