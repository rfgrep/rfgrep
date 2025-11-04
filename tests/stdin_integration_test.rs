use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn target_debug() -> PathBuf {
    let mut p = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    p.push("target/debug/rfgrep");
    p
}

#[test]
fn test_stdin_basic_search() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(target_debug())
        .arg("search")
        .arg("error")
        .write_stdin("line 1 with error\nline 2 ok\nline 3 error again\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("error"));

    Ok(())
}

#[test]
fn test_stdin_count_mode() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(target_debug())
        .arg("search")
        .arg("test")
        .arg("-c")
        .write_stdin("test line 1\ntest line 2\ntest line 3\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));

    Ok(())
}

#[test]
fn test_stdin_files_with_matches() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(target_debug())
        .arg("search")
        .arg("pattern")
        .arg("-l")
        .write_stdin("line with pattern\nother line\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("<stdin>"));

    Ok(())
}

#[test]
fn test_stdin_case_sensitive() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(target_debug())
        .arg("search")
        .arg("Test")
        .arg("--case-sensitive")
        .write_stdin("test\nTest\nTEST\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 1"));

    Ok(())
}

#[test]
fn test_stdin_max_matches() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(target_debug())
        .arg("search")
        .arg("x")
        .arg("--max-matches")
        .arg("2")
        .write_stdin("x\nx\nx\nx\nx\n")
        .assert()
        .success();
    // Should stop after 2 matches

    Ok(())
}

#[test]
fn test_stdin_empty_input() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(target_debug())
        .arg("search")
        .arg("pattern")
        .write_stdin("")
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_stdin_no_matches() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(target_debug())
        .arg("search")
        .arg("notfound")
        .write_stdin("some text without the pattern\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));

    Ok(())
}

#[test]
fn test_output_format_json() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content\n")?;

    Command::new(target_debug())
        .arg("search")
        .arg("test")
        .arg("--output-format")
        .arg("json")
        .arg("--")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"line_number\""));

    Ok(())
}

#[test]
fn test_output_format_csv() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content\n")?;

    Command::new(target_debug())
        .arg("search")
        .arg("test")
        .arg("--output-format")
        .arg("csv")
        .arg("--")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("File,Line"));

    Ok(())
}

#[test]
fn test_output_format_ndjson() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content\n")?;

    Command::new(target_debug())
        .arg("search")
        .arg("test")
        .arg("--ndjson")
        .arg("--")
        .arg(temp_dir.path())
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_search_algorithms() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content for algorithm testing\n")?;

    // Test Boyer-Moore
    Command::new(target_debug())
        .arg("search")
        .arg("test")
        .arg("--algorithm")
        .arg("boyer-moore")
        .arg("--")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Test Regex
    Command::new(target_debug())
        .arg("search")
        .arg("test")
        .arg("--algorithm")
        .arg("regex")
        .arg("--")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Test Simple
    Command::new(target_debug())
        .arg("search")
        .arg("test")
        .arg("--algorithm")
        .arg("simple")
        .arg("--")
        .arg(temp_dir.path())
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_quiet_mode() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content\n")?;

    let output = Command::new(target_debug())
        .arg("search")
        .arg("test")
        .arg("--quiet")
        .arg("--")
        .arg(temp_dir.path())
        .assert()
        .success()
        .get_output()
        .clone();

    // Quiet mode should not show "Searching N files"
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Searching"));

    Ok(())
}

#[test]
fn test_multiple_extensions() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    fs::write(temp_dir.path().join("test1.txt"), "content")?;
    fs::write(temp_dir.path().join("test2.rs"), "content")?;
    fs::write(temp_dir.path().join("test3.md"), "content")?;
    fs::write(temp_dir.path().join("test4.py"), "content")?;

    Command::new(target_debug())
        .arg("search")
        .arg("content")
        .arg("--extensions")
        .arg("txt,rs")
        .arg("--")
        .arg(temp_dir.path())
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_recursive_search() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir)?;
    fs::write(temp_dir.path().join("top.txt"), "top level")?;
    fs::write(subdir.join("nested.txt"), "nested level")?;

    Command::new(target_debug())
        .arg("search")
        .arg("level")
        .arg("--recursive")
        .arg("--")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 2"));

    Ok(())
}

#[test]
fn test_context_lines() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "line 1\nline 2\nMATCH\nline 4\nline 5\n")?;

    Command::new(target_debug())
        .arg("search")
        .arg("MATCH")
        .arg("--context-lines")
        .arg("2")
        .arg("--")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("line 2"))
        .stdout(predicate::str::contains("line 4"));

    Ok(())
}

#[test]
fn test_invert_match() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "keep\nremove\nkeep\nremove\n")?;

    Command::new(target_debug())
        .arg("search")
        .arg("remove")
        .arg("--invert-match")
        .arg("--")
        .arg(temp_dir.path())
        .assert()
        .success();
    // Should only show lines without "remove"

    Ok(())
}
