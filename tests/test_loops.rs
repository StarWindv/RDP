use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_while_loop() {
    let mut cmd = Command::cargo_bin("rs-dash-pro").unwrap();
    cmd.arg("-c")
        .arg("i=0; while [ $i -lt 3 ]; do echo $i; i=$((i+1)); done");
    
    cmd.assert().success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"));
}

#[test]
fn test_for_loop() {
    let mut cmd = Command::cargo_bin("rs-dash-pro").unwrap();
    cmd.arg("-c")
        .arg("for i in a b c; do echo $i; done");
    
    cmd.assert().success()
        .stdout(predicate::str::contains("a"))
        .stdout(predicate::str::contains("b"))
        .stdout(predicate::str::contains("c"));
}

#[test]
fn test_nested_loops() {
    let mut cmd = Command::cargo_bin("rs-dash-pro").unwrap();
    cmd.arg("-c")
        .arg("for i in 1 2; do for j in a b; do echo $i$j; done; done");
    
    cmd.assert().success()
        .stdout(predicate::str::contains("1a"))
        .stdout(predicate::str::contains("1b"))
        .stdout(predicate::str::contains("2a"))
        .stdout(predicate::str::contains("2b"));
}
