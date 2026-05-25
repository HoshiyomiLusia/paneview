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
- Built-in version check and self-update command.

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
- Rust 1.95 or newer.
- Cargo.
- Git.
- A UTF-8 capable terminal.
- A local Unix shell such as `zsh`, `bash`, or `sh`.

Cargo downloads Rust crate dependencies automatically during installation or builds. The main crates are `ratatui`, `crossterm`, `portable-pty`, `sysinfo`, `if-addrs`, `vt100`, `crossbeam-channel`, and `anyhow`.

### macOS

Check the environment:

```bash
rustc --version
cargo --version
git --version
echo "$PATH" | tr ':' '\n' | grep -x "$HOME/.cargo/bin"
```

Install Rust and Cargo if they are missing:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Install Git if it is missing:

```bash
xcode-select --install
```

### Linux

Check the environment:

```bash
rustc --version
cargo --version
git --version
echo "$PATH" | tr ':' '\n' | grep -x "$HOME/.cargo/bin"
```

Install Rust and Cargo if they are missing:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Install Git with your distribution package manager. For Debian or Ubuntu:

```bash
sudo apt update
sudo apt install -y git
```

### Windows

Windows is not a supported target for this MVP. Use macOS or Linux, or run PaneView in a Linux environment with Rust, Cargo, Git, and a Unix shell available.

## Project Installation

Install PaneView after the environment dependencies are ready:

```bash
cargo install --git https://github.com/HoshiyomiLusia/paneview.git --locked
```

This installs the `paneview` command into Cargo's binary directory, usually `$HOME/.cargo/bin`.

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
cargo uninstall paneview
```

## Usage

Run the TUI:

```bash
paneview
```

Show the installed version and build commit:

```bash
paneview --version
```

Check whether the installed build matches the latest GitHub `main` commit:

```bash
paneview check-update
```

Update PaneView through Cargo:

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
