use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_while_loop_simple() {
    let mut cmd = Command::cargo_bin("rs-dash-pro").unwrap();
    // while loop that doesn't need arithmetic
    cmd.arg("-c").arg("i=1; while [ 1 ]; do echo $i; break; done");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

#[test]
fn test_while_loop_false() {
    let mut cmd = Command::cargo_bin("rs-dash-pro").unwrap();
    // while loop that should not execute
    cmd.arg("-c").arg("while false; do echo should_not_print; done; echo done");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("done"));
}

#[test]
fn test_until_loop_simple() {
    let mut cmd = Command::cargo_bin("rs-dash-pro").unwrap();
    // until loop
    cmd.arg("-c").arg("until true; do echo should_not_print; done; echo done");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("done"));
}
