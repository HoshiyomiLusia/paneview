# PaneView

PaneView is a cross-platform Rust TUI for split PTY-backed shell panes and local system monitoring.

## One-line install

If Rust and Cargo are already installed:

```bash
cargo install --git https://github.com/HoshiyomiLusia/paneview.git --locked
```

Then run:

```bash
paneview
```

## Requirements

- macOS or Linux
- Rust 1.85+ with Cargo
- `$HOME/.cargo/bin` in your `PATH`
- A local shell such as `zsh`, `bash`, or `sh`
- A UTF-8 capable terminal

No root permission is required.

## Install Rust

If Rust is not installed yet:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Load Cargo into your current shell:

```bash
source "$HOME/.cargo/env"
```

Check it:

```bash
cargo --version
```

## Project dependencies

Rust crate dependencies are installed automatically by Cargo. You do not need to install them manually.

Main crates:

- `ratatui` for TUI rendering
- `crossterm` for terminal input and screen control
- `portable-pty` for PTY-backed shell panes
- `sysinfo` for system metrics
- `if-addrs` for network interface addresses
- `vt100` for terminal output parsing
- `crossbeam-channel` for PTY output communication
- `anyhow` for error handling

## Build from source

```bash
git clone https://github.com/HoshiyomiLusia/paneview.git
cd paneview
cargo build --release
```

Run the release binary:

```bash
./target/release/paneview
```

Install from the local checkout:

```bash
cargo install --path . --locked
```

## Features

- Multiple split terminal panes
- Vertical and horizontal splits
- PTY-backed shell execution
- Focus switching between panes
- Input forwarding to the focused pane
- `Ctrl+C` sent to the focused pane process
- Toggleable graphical system status panel
- CPU, memory, disk, network, OS, kernel, hostname, and uptime display

## Keybindings

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
| `Ctrl+S` | Toggle the system status panel |
| `Ctrl+C` | Send interrupt to the focused pane process |

## Development

```bash
cargo fmt
cargo check
cargo test
```

## Limitations

- This is an MVP, not a tmux replacement.
- Pane rendering uses `vt100` for common ANSI output, but it is not a complete terminal emulator.
- Each pane starts the user's default shell.
- Closing the last pane is blocked.
- GPU, temperature, fan, packet capture, remote host management, and plugins are not included.
- Unavailable system metrics are shown as `N/A`.
