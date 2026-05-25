use std::process::{Command, Stdio};

use anyhow::{Context, bail};

const REPOSITORY: &str = "https://github.com/HoshiyomiLusia/paneview.git";
const MAIN_REF: &str = "refs/heads/main";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliAction {
    RunTui,
    Exit,
}

pub fn handle_args() -> anyhow::Result<CliAction> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [] => Ok(CliAction::RunTui),
        [arg] if matches!(arg.as_str(), "-h" | "--help" | "help") => {
            print_help();
            Ok(CliAction::Exit)
        }
        [arg] if matches!(arg.as_str(), "-V" | "--version" | "version") => {
            println!("{}", version_string());
            Ok(CliAction::Exit)
        }
        [arg] if arg == "check-update" => {
            check_update()?;
            Ok(CliAction::Exit)
        }
        [arg] if arg == "update" => {
            update()?;
            Ok(CliAction::Exit)
        }
        _ => {
            print_help();
            bail!("unknown arguments: {}", args.join(" "));
        }
    }
}

fn print_help() {
    println!(
        "\
{}

Usage:
  paneview
  paneview --version
  paneview check-update
  paneview update

Commands:
  check-update   Compare this build with the latest GitHub main commit.
  update         Reinstall PaneView from GitHub using Cargo.

Options:
  -h, --help     Show this help.
  -V, --version  Show version and build commit.
",
        version_string()
    );
}

fn check_update() -> anyhow::Result<()> {
    let local = git_hash();
    println!("Current: {}", version_string());

    let remote = remote_main_hash().context("failed to check remote main branch")?;
    let remote_short = short_hash(&remote);
    println!("Latest:  {} ({remote_short})", env!("CARGO_PKG_VERSION"));

    if local != "unknown" && remote.starts_with(local) {
        println!("PaneView is up to date.");
    } else {
        println!("Update available or local build is not from the latest main commit.");
        println!("Run: paneview update");
    }

    Ok(())
}

fn update() -> anyhow::Result<()> {
    println!("Updating PaneView from {REPOSITORY}");
    let status = Command::new("cargo")
        .args(["install", "--git", REPOSITORY, "--locked", "--force"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to run cargo install")?;

    if !status.success() {
        bail!("cargo install failed with status {status}");
    }

    println!("PaneView update completed.");
    Ok(())
}

fn remote_main_hash() -> anyhow::Result<String> {
    let output = Command::new("git")
        .args(["ls-remote", REPOSITORY, MAIN_REF])
        .output()
        .context("failed to run git ls-remote")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git ls-remote failed: {}", stderr.trim());
    }

    let stdout = String::from_utf8(output.stdout).context("git output was not valid UTF-8")?;
    let hash = stdout
        .split_whitespace()
        .next()
        .filter(|value| !value.is_empty())
        .context("remote main branch was not found")?;

    Ok(hash.to_string())
}

fn version_string() -> String {
    format!("paneview {} ({})", env!("CARGO_PKG_VERSION"), git_hash())
}

fn git_hash() -> &'static str {
    option_env!("PANEVIEW_GIT_HASH").unwrap_or("unknown")
}

fn short_hash(hash: &str) -> String {
    hash.chars().take(12).collect()
}
