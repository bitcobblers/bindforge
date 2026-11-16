use std::process::Command;

#[test]
fn test_cli_list_format() {
    let output = Command::new("cargo")
        .args(["run", "-p", "discovery-cli", "--", "list"])
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("| Slug | Display Name | Install Path | Platform |"));
}

#[test]
fn test_cli_list_json_format() {
    let output = Command::new("cargo")
        .args(["run", "-p", "discovery-cli", "--", "list", "--json"])
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "[]");
}
