//! What the README promises can be run: every `cargo run --release -- ...` line of the quick start works on the repository, and the help runs.

use std::path::PathBuf;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_eh3d-launcher");

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn of_readme() -> Vec<Vec<String>> {
    std::fs::read_to_string(root().join("README.md"))
        .expect("the README is there")
        .lines()
        .filter_map(|l| l.trim().strip_prefix("cargo run --release -- "))
        .map(|rest| rest.split_whitespace().map(str::to_string).collect())
        .collect()
}

#[test]
fn the_commands_of_the_readme_run() {
    let commands = of_readme();
    assert!(!commands.is_empty(), "the README shows no command");
    for args in commands {
        let out = Command::new(BIN)
            .args(&args)
            .current_dir(root())
            .output()
            .expect("the program runs");
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn the_help_runs_and_names_the_commands() {
    let out = Command::new(BIN)
        .arg("help")
        .output()
        .expect("the program runs");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    for command in [
        "list",
        "launch",
        "new-project",
        "add-layer",
        "verify",
        "recent",
    ] {
        assert!(text.contains(command), "the help does not name {command}");
    }
}
