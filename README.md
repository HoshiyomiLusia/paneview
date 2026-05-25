# PaneView

## Project Overview

PaneView is a cross-platform Rust terminal UI for split PTY-backed shell panes and local system monitoring.

It is built as a lightweight MVP for macOS and Linux. It gives one terminal window multiple shell panes, a graphical system dashboard, and simple keyboard controls. It is not intended to replace tmux.

Supported features:

- Split terminal panes backed by real PTYs.
- Vertical and horizontal pane splits.
- Keyboard focus switching between panes.
- Input forwarding to the focused pane.
- `Ctrl+C` delivery to the focused pane process.
- Toggleable system dashboard with CPU, memory, disk, network, OS, kernel, hostname, and uptime data.
- Built-in version check and release-based self-update command.

### Example

TODO: Add a sample image.

```text
+---------------------------------------------------------------+
| CPU / Memory / Network / Disk / Interfaces / System Dashboard |
+-------------------------------+-------------------------------+
| pane 1                        | pane 2                        |
| $                             | $                             |
+-------------------------------+-------------------------------+
| mode:normal | pane:1/2 | Ctrl+Q quit | Ctrl+S dashboard       |
+---------------------------------------------------------------+
```

## Environment Dependencies

PaneView requires:

- macOS or Linux.
- `curl`.
- `tar`.
- A UTF-8 capable terminal.
- A local Unix shell such as `zsh`, `bash`, or `sh`.

Prebuilt release targets:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

Rust, Cargo, and Git are only required when building from source. Cargo downloads Rust crate dependencies automatically during source builds. The main crates are `ratatui`, `crossterm`, `portable-pty`, `sysinfo`, `if-addrs`, `vt100`, `crossbeam-channel`, and `anyhow`.

### macOS

Check the environment:

```bash
uname -s
uname -m
curl --version
tar --version
echo "$PATH" | tr ':' '\n' | grep -E '(/usr/local/bin|/opt/homebrew/bin|/\.local/bin)$'
```

Install command line tools if `curl` or `tar` is missing:

```bash
xcode-select --install
```

For source builds, also install Rust and Git:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
git --version || xcode-select --install
```

### Linux

Check the environment:

```bash
uname -s
uname -m
curl --version
tar --version
echo "$PATH" | tr ':' '\n' | grep -E '(/usr/local/bin|/\.local/bin)$'
```

Install `curl` and `tar` with your distribution package manager. For Debian or Ubuntu:

```bash
sudo apt update
sudo apt install -y curl tar
```

For source builds, also install Rust and Git:

```bash
sudo apt install -y git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### Windows

Windows is not a supported target for this MVP. Use macOS or Linux, or run PaneView in a Linux environment with `curl`, `tar`, and a Unix shell available.

## Project Installation

Install the latest prebuilt release:

```bash
curl -fsSL https://raw.githubusercontent.com/HoshiyomiLusia/paneview/main/install.sh | sh
```

The installer downloads a GitHub Release archive, verifies its checksum when possible, and installs the `paneview` binary. It uses `/usr/local/bin` when that directory is writable; otherwise it uses `$HOME/.local/bin`.

Install to a custom directory:

```bash
PANEVIEW_INSTALL_DIR="$HOME/bin" sh -c 'curl -fsSL https://raw.githubusercontent.com/HoshiyomiLusia/paneview/main/install.sh | sh'
```

Install a specific release:

```bash
PANEVIEW_VERSION="v0.1.2" sh -c 'curl -fsSL https://raw.githubusercontent.com/HoshiyomiLusia/paneview/main/install.sh | sh'
```

Build and install from source:

```bash
cargo install --git https://github.com/HoshiyomiLusia/paneview.git --locked
```

Build from a local checkout:

```bash
git clone https://github.com/HoshiyomiLusia/paneview.git
cd paneview
cargo build --release
```

Install from a local checkout:

```bash
cargo install --path . --locked --force
```

Uninstall:

```bash
rm -f "$HOME/.local/bin/paneview"
```

If PaneView was installed with Cargo, uninstall it with `cargo uninstall paneview`.

## Usage

Run the TUI:

```bash
paneview
```

Show the installed version and build commit:

```bash
paneview --version
```

Check whether the installed build matches the latest GitHub release:

```bash
paneview check-update
```

Update PaneView from the latest prebuilt GitHub release:

```bash
paneview update
```

Development checks:

```bash
cargo fmt
cargo check
cargo test
```

Keybindings:

| Key | Action |
| --- | --- |
| `Ctrl+Q` | Quit PaneView |
| `Ctrl+H` | Focus pane on the left |
| `Ctrl+J` | Focus pane below |
| `Ctrl+K` | Focus pane above |
| `Ctrl+L` | Focus pane on the right |
| `Ctrl+\` | Create a vertical split |
| `Ctrl+-` | Create a horizontal split |
| `Ctrl+N` | Create a new pane with a vertical split |
| `Ctrl+W` | Close the focused pane |
| `Ctrl+S` | Toggle the dashboard |
| `Ctrl+C` | Send interrupt to the focused pane process |

Known limitations:

- PaneView is an MVP.
- Pane rendering uses `vt100` for common ANSI output, but it is not a complete terminal emulator.
- Each pane starts the user's default shell.
- Closing the last pane is blocked.
- Mouse-clickable buttons are not implemented.
- GPU, temperature, fan, packet capture, remote host management, and plugins are not included.
- Unavailable system metrics are shown as `N/A`.
