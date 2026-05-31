//! Command-line argument handling plus the `check-update` / `update`
//! subcommands.
//!
//! Both subcommands shell out to `curl` (and `tar` for `update`) rather
//! than pulling in an HTTP client — the install script already requires
//! these tools and they're present on every supported platform.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, bail};

const REPO_SLUG: &str = "HoshiyomiLusia/paneview";
const LATEST_RELEASE_URL: &str = "https://github.com/HoshiyomiLusia/paneview/releases/latest";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliAction {
    RunTui,
    Exit,
}

pub fn handle_args() -> anyhow::Result<CliAction> {
    let args = env::args().skip(1).collect::<Vec<_>>();
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
  update         Download and install the latest GitHub release binary.

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

    if tag_matches_current(&latest) {
        println!("PaneView is up to date.");
    } else {
        println!("Update available.");
        println!("Run: paneview update");
    }

    Ok(())
}

fn update() -> anyhow::Result<()> {
    println!("Checking GitHub for the latest release.");
    let latest = latest_release_tag().context("failed to query latest GitHub release")?;
    println!("Latest:  {latest}");
    println!("Current: {}", version_string());

    if tag_matches_current(&latest) {
        println!("PaneView is already up to date.");
        return Ok(());
    }

    let target = detect_target().context("could not determine release target for this platform")?;
    let asset = format!("paneview-{target}.tar.gz");
    let base_url = format!(
        "https://github.com/{REPO_SLUG}/releases/download/{tag}/",
        tag = latest
    );
    let archive_url = format!("{base_url}{asset}");
    let checksum_url = format!("{archive_url}.sha256");

    let tmp = tempdir().context("failed to create staging directory")?;
    let archive_path = tmp.path().join(&asset);
    let checksum_path = tmp.path().join(format!("{asset}.sha256"));

    println!("Downloading {archive_url}");
    download(&archive_url, &archive_path).with_context(|| format!("downloading {archive_url}"))?;
    println!("Downloading checksum");
    download(&checksum_url, &checksum_path)
        .with_context(|| format!("downloading {checksum_url} (release missing .sha256)"))?;

    verify_checksum(&archive_path, &checksum_path)
        .with_context(|| format!("checksum verification failed for {asset}"))?;
    println!("Checksum OK.");

    extract_tar_gz(&archive_path, tmp.path())
        .with_context(|| format!("failed to extract {asset}"))?;
    let new_binary = tmp.path().join("paneview");
    if !new_binary.is_file() {
        bail!("extracted archive did not contain a paneview binary");
    }
    set_executable(&new_binary)?;

    let target_path = env::current_exe().context("failed to locate the running binary path")?;
    replace_binary(&new_binary, &target_path).with_context(|| {
        format!(
            "failed to replace running binary at {}",
            target_path.display()
        )
    })?;

    println!("Installed paneview at {}", target_path.display());
    println!("Run `paneview --version` in a fresh shell to confirm.");
    Ok(())
}

fn tag_matches_current(tag: &str) -> bool {
    tag.trim_start_matches('v') == env!("CARGO_PKG_VERSION")
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

fn detect_target() -> anyhow::Result<&'static str> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    Ok(match (os, arch) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        (os, arch) => bail!("unsupported platform: {os}/{arch}"),
    })
}

fn download(url: &str, dest: &Path) -> anyhow::Result<()> {
    let status = Command::new("curl")
        .args(["-fL", url, "-o"])
        .arg(dest)
        .status()
        .context("failed to invoke curl")?;
    if !status.success() {
        bail!("curl exited with status {status}");
    }
    Ok(())
}

fn verify_checksum(archive: &Path, checksum_file: &Path) -> anyhow::Result<()> {
    let expected_line = fs::read_to_string(checksum_file)
        .with_context(|| format!("reading {}", checksum_file.display()))?;
    let expected = expected_line
        .split_whitespace()
        .next()
        .filter(|hex| hex.len() == 64)
        .context("checksum file did not contain a 64-char SHA-256 hex")?
        .to_lowercase();

    let actual = compute_sha256(archive)?;

    if expected != actual {
        bail!("expected {expected}, got {actual}");
    }
    Ok(())
}

fn compute_sha256(path: &Path) -> anyhow::Result<String> {
    // Prefer sha256sum (Linux); fall back to shasum (BSD/macOS).
    let (program, args) = if which("sha256sum").is_some() {
        ("sha256sum", vec![])
    } else if which("shasum").is_some() {
        ("shasum", vec!["-a", "256"])
    } else {
        bail!("neither sha256sum nor shasum is available");
    };

    let output = Command::new(program)
        .args(args)
        .arg(path)
        .output()
        .with_context(|| format!("failed to run {program}"))?;
    if !output.status.success() {
        bail!(
            "{program} exited with {status}: {stderr}",
            status = output.status,
            stderr = String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let stdout = String::from_utf8(output.stdout).context("checksum output not UTF-8")?;
    let hex = stdout
        .split_whitespace()
        .next()
        .filter(|hex| hex.len() == 64)
        .context("could not parse checksum output")?;
    Ok(hex.to_lowercase())
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> anyhow::Result<()> {
    let status = Command::new("tar")
        .arg("-xzf")
        .arg(archive)
        .arg("-C")
        .arg(dest)
        .status()
        .context("failed to invoke tar")?;
    if !status.success() {
        bail!("tar exited with status {status}");
    }
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)
        .with_context(|| format!("statting {}", path.display()))?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).with_context(|| format!("chmod 755 {}", path.display()))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}

/// Atomically swap `new` into `target`. On Unix this works even when
/// `target` is the currently-running binary: the kernel keeps the open
/// inode alive while readers retain references.
///
/// We try `fs::rename` first (cheap, atomic if same FS). If that fails
/// because the staging dir is on a different filesystem, fall back to
/// copying via a sibling temp file and renaming.
fn replace_binary(new: &Path, target: &Path) -> anyhow::Result<()> {
    if fs::rename(new, target).is_ok() {
        return Ok(());
    }

    // Fallback: stage next to the target, then rename.
    let parent = target
        .parent()
        .with_context(|| format!("{} has no parent directory", target.display()))?;
    let pid = std::process::id();
    let staging = parent.join(format!(".paneview.tmp.{pid}"));
    fs::copy(new, &staging).with_context(|| format!("copying to {}", staging.display()))?;
    set_executable(&staging)?;
    fs::rename(&staging, target)
        .with_context(|| format!("rename {} -> {}", staging.display(), target.display()))?;
    Ok(())
}

fn which(program: &str) -> Option<PathBuf> {
    let output = Command::new("sh")
        .args(["-c", &format!("command -v {program}")])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

fn version_string() -> String {
    format!("paneview {} ({})", env!("CARGO_PKG_VERSION"), git_hash())
}

fn git_hash() -> &'static str {
    option_env!("PANEVIEW_GIT_HASH").unwrap_or("unknown")
}

/// RAII temp directory. Removed on drop. Uses `mktemp -d` since std
/// doesn't provide a temp dir API without an extra crate.
struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // Best-effort cleanup.
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn tempdir() -> anyhow::Result<TempDir> {
    let output = Command::new("mktemp")
        .args(["-d", "-t", "paneview-update.XXXXXX"])
        .output()
        .context("mktemp not available")?;
    if !output.status.success() {
        bail!(
            "mktemp failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let path = PathBuf::from(
        String::from_utf8(output.stdout)
            .context("mktemp output not UTF-8")?
            .trim(),
    );
    if !path.is_dir() {
        bail!("mktemp returned non-directory: {}", path.display());
    }
    Ok(TempDir { path })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_matches_current_strips_v_prefix() {
        let current_version = env!("CARGO_PKG_VERSION");
        assert!(tag_matches_current(&format!("v{current_version}")));
        assert!(tag_matches_current(current_version));
        assert!(!tag_matches_current("v0.0.0"));
    }

    #[test]
    fn detect_target_returns_supported_triple() {
        // Only assert it doesn't bail on our test runners (macOS/Linux).
        let target = detect_target().expect("supported platform");
        assert!(target.contains("apple-darwin") || target.contains("unknown-linux-gnu"));
    }
}
