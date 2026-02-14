use assert_cmd::Command;
use predicates::prelude::*;

#[allow(deprecated)]
#[test]
fn worker_timeout_enforced() {
    let mut cmd = Command::cargo_bin("rfgrep").unwrap();
    cmd.env("RFGREP_WORKER_SLEEP", "3");
    cmd.arg("search")
        .arg("pattern")
        .arg("--timeout-per-file")
        .arg("1")
        .arg("--")
        .arg("bench_data/file1.txt");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}
