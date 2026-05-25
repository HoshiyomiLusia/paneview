use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");

    let mut hash = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|hash| hash.trim().to_string())
        .filter(|hash| !hash.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    if hash != "unknown" && is_dirty() {
        hash.push_str("-dirty");
    }

    println!("cargo:rustc-env=PANEVIEW_GIT_HASH={hash}");
}

fn is_dirty() -> bool {
    Command::new("git")
        .args(["diff", "--quiet", "HEAD", "--"])
        .status()
        .map(|status| !status.success())
        .unwrap_or(false)
}
