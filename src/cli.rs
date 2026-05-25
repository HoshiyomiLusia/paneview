use std::process::{Command, Stdio};

use anyhow::{Context, bail};

const LATEST_RELEASE_URL: &str = "https://github.com/HoshiyomiLusia/paneview/releases/latest";
const INSTALL_SCRIPT_URL: &str =
    "https://raw.githubusercontent.com/HoshiyomiLusia/paneview/main/install.sh";

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
  check-update   Compare this build with the latest GitHub release.
  update         Install the latest GitHub release binary.

Options:
  -h, --help     Show this help.
  -V, --version  Show version and build commit.
",
        version_string()
    );
}

fn check_update() -> anyhow::Result<()> {
    println!("Current: {}", version_string());

    let latest = latest_release_tag().context("failed to check latest GitHub release")?;
    println!("Latest:  {latest}");

    if latest.trim_start_matches('v') == env!("CARGO_PKG_VERSION") {
        println!("PaneView is up to date.");
    } else {
        println!("Update available.");
        println!("Run: paneview update");
    }

    Ok(())
}

fn update() -> anyhow::Result<()> {
    println!("Updating PaneView from the latest GitHub release.");
    let command = format!("curl -fsSL {INSTALL_SCRIPT_URL} | sh");
    let status = Command::new("sh")
        .args(["-c", &command])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to run install script")?;

    if !status.success() {
        bail!("install script failed with status {status}");
    }

    println!("PaneView update completed.");
    Ok(())
}

fn latest_release_tag() -> anyhow::Result<String> {
    let output = Command::new("curl")
        .args([
            "-fsSLo",
            "/dev/null",
            "-w",
            "%{url_effective}",
            LATEST_RELEASE_URL,
        ])
        .output()
        .context("failed to run curl")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("curl failed: {}", stderr.trim());
    }

    let effective_url =
        String::from_utf8(output.stdout).context("curl output was not valid UTF-8")?;
    let tag = effective_url
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|value| value.starts_with('v'))
        .context("latest release tag was not found")?;

    Ok(tag.to_string())
}

fn version_string() -> String {
    format!("paneview {} ({})", env!("CARGO_PKG_VERSION"), git_hash())
}

fn git_hash() -> &'static str {
    option_env!("PANEVIEW_GIT_HASH").unwrap_or("unknown")
}
