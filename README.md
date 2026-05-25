# PaneView

PaneView is a cross-platform Rust terminal UI for running PTY-backed shell panes and monitoring local system status.

It targets macOS and Linux. The first version is an MVP, not a full tmux replacement.

## Features

- Split terminal panes in a TUI.
- PTY-backed shell execution for each pane.
- Vertical and horizontal splits.
- Focus switching between panes.
- Input forwarding to the focused pane.
- `Ctrl+C` is sent to the focused pane process instead of closing PaneView.
- Close the focused pane.
- Toggle a graphical system status panel.
- Show CPU, memory, disk, network interfaces, IP addresses, network throughput, OS name, kernel version, hostname, and uptime.

## Dependencies

### System Requirements

- macOS or Linux.
- A terminal emulator with UTF-8 support.
- Rust toolchain with Cargo.
- A local shell such as `zsh`, `bash`, or `sh`.

No root permission is required.

### Install Rust

Install Rust with `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then restart your terminal or load Cargo into your current shell:

```bash
source "$HOME/.cargo/env"
```

Check the installation:

```bash
rustc --version
cargo --version
```

### Rust Crates

Cargo installs Rust crate dependencies automatically when you build or install the project.

Main crates used by PaneView:

- `ratatui`: terminal UI rendering.
- `crossterm`: terminal input and screen control.
- `portable-pty`: cross-platform PTY support.
- `sysinfo`: system metrics.
- `if-addrs`: network interface addresses.
- `vt100`: terminal output parsing.
- `crossbeam-channel`: PTY reader communication.
- `anyhow`: error handling.

You do not need to install these crates manually.

## Build

```bash
cargo build
```

For an optimized binary:

```bash
cargo build --release
```

## Run

Run from the project directory:

```bash
cargo run
```

Or run the debug binary:

```bash
./target/debug/paneview
```

Or run the release binary:

```bash
./target/release/paneview
```

## Install as a Command

Install PaneView into Cargo's binary directory:

```bash
cargo install --path .
```

Make sure Cargo's binary directory is in your `PATH`:

```bash
echo "$PATH"
```

It should include:

```text
$HOME/.cargo/bin
```

After installation, run:

```bash
paneview
```

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

## Project Structure

```text
src/
  app.rs      Application state, pane management, focus management
  input.rs    Keyboard input handling
  layout.rs   Split tree layout and layout tests
  main.rs     Terminal setup and main loop
  pane.rs     PTY, shell process, output buffer
  system.rs   macOS/Linux system information collection
  tui.rs      ratatui rendering
```

## Known Limitations

- PaneView is an MVP.
- The pane renderer uses `vt100` for common ANSI output parsing, but it is not a complete terminal emulator.
- Each pane starts the user's default shell. There is no command palette yet.
- Closing the last pane is blocked.
- GPU monitoring, temperature, fan speed, packet capture, remote host management, and plugins are not included.
- Metrics that are unavailable on a platform are shown as `N/A`.

## Development Checks

```bash
cargo fmt
cargo check
cargo test
```
