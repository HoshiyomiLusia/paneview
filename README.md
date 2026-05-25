# PaneView

PaneView is a cross-platform Rust TUI for split PTY-backed shell panes and local system monitoring.

It is an MVP for macOS and Linux. It is not intended to replace tmux.

## Install

If Rust and Cargo are already installed:

```bash
cargo install --git https://github.com/HoshiyomiLusia/paneview.git --locked
```

Run it:

```bash
paneview
```

Update an existing install:

```bash
cargo install --git https://github.com/HoshiyomiLusia/paneview.git --locked --force
```

Uninstall:

```bash
cargo uninstall paneview
```

## Version and Updates

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

The update command runs:

```bash
cargo install --git https://github.com/HoshiyomiLusia/paneview.git --locked --force
```

## Requirements

- macOS or Linux
- Rust 1.95+ with Cargo
- `$HOME/.cargo/bin` in your `PATH`
- A UTF-8 capable terminal
- A local shell such as `zsh`, `bash`, or `sh`
- `git` in your `PATH` for `paneview check-update`

No root permission is required.

If Rust is missing, install it with rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## Dependencies

PaneView has no extra system package dependency beyond a working Rust toolchain on macOS/Linux.

Cargo installs Rust crate dependencies automatically during `cargo install` or `cargo build`.

Main crates:

- `ratatui`: TUI rendering
- `crossterm`: terminal input and screen control
- `portable-pty`: PTY-backed shell panes
- `sysinfo`: CPU, memory, disk, network, and OS metrics
- `if-addrs`: network interface addresses
- `vt100`: ANSI terminal output parsing
- `crossbeam-channel`: PTY output communication
- `anyhow`: error handling

## Build From Source

```bash
git clone https://github.com/HoshiyomiLusia/paneview.git
cd paneview
cargo build --release
```

Run the local release binary:

```bash
./target/release/paneview
```

Install from the local checkout:

```bash
cargo install --path . --locked --force
```

## Features

- Multiple split terminal panes
- Vertical and horizontal pane splits
- PTY-backed shell execution
- Keyboard focus switching between panes
- Input forwarding to the focused pane
- `Ctrl+C` sent to the focused pane process
- Toggleable top dashboard with system status cards
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
| `Ctrl+S` | Toggle the dashboard |
| `Ctrl+C` | Send interrupt to the focused pane process |

## Layout

PaneView uses a simple three-part TUI layout:

```text
dashboard
terminal panes
footer
```

The dashboard is hidden automatically in very small terminal windows.

## Development

```bash
cargo fmt
cargo check
cargo test
```

## Limitations

- This is an MVP.
- Pane rendering uses `vt100` for common ANSI output, but it is not a complete terminal emulator.
- Each pane starts the user's default shell.
- Closing the last pane is blocked.
- Mouse-clickable buttons are not implemented yet.
- GPU, temperature, fan, packet capture, remote host management, and plugins are not included.
- Unavailable system metrics are shown as `N/A`.
