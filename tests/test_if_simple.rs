use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_if_true() {
    let mut cmd = Command::cargo_bin("rs-dash-pro").unwrap();
    cmd.arg("-c")
        .arg("if true; then echo yes; fi");
    
    cmd.assert().success()
        .stdout(predicate::str::contains("yes"));
}

#[test]
fn test_if_false() {
    let mut cmd = Command::cargo_bin("rs-dash-pro").unwrap();
    cmd.arg("-c")
        .arg("if false; then echo yes; else echo no; fi");
    
    cmd.assert().success()
        .stdout(predicate::str::contains("no"));
}

#[test]
fn test_if_elif_else() {
    let mut cmd = Command::cargo_bin("rs-dash-pro").unwrap();
    cmd.arg("-c")
        .arg("if false; then echo a; elif true; then echo b; else echo c; fi");
    
    cmd.assert().success()
        .stdout(predicate::str::contains("b"));
}
