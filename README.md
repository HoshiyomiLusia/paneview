# PaneView

## Project Overview

PaneView is a Rust terminal UI for PTY-backed split shell panes and local system monitoring on macOS and Linux.

![PaneView example](docs/demo-preview.png)

PaneView provides:

- Interactive shell panes backed by real PTYs.
- Vertical and horizontal splits.
- CPU, memory, disk, network, interface, host, kernel, and uptime views.
- Release-based install, update, and version commands.

## Environment Dependencies

PaneView is distributed as prebuilt binaries. Normal installation does not require Rust, Cargo, Git, a compiler, or a source checkout.

Supported release targets:

- macOS Apple Silicon: `aarch64-apple-darwin`
- macOS Intel: `x86_64-apple-darwin`
- Linux ARM64: `aarch64-unknown-linux-gnu`
- Linux x86_64: `x86_64-unknown-linux-gnu`

Windows is not supported.

The one-line installer uses `curl` and `tar` to download and extract the matching release archive. If those tools are missing, install them with your system package manager or download the archive manually from the GitHub Releases page.

## Project Installation

Install the latest release:

```bash
curl -fsSL https://raw.githubusercontent.com/HoshiyomiLusia/paneview/main/install.sh | sh
```

Run:

```bash
paneview
```

Install a specific version:

```bash
PANEVIEW_VERSION="v0.1.6" sh -c 'curl -fsSL https://raw.githubusercontent.com/HoshiyomiLusia/paneview/main/install.sh | sh'
```

Install to a custom directory:

```bash
PANEVIEW_INSTALL_DIR="$HOME/bin" sh -c 'curl -fsSL https://raw.githubusercontent.com/HoshiyomiLusia/paneview/main/install.sh | sh'
```

Build from source only if you are developing PaneView:

```bash
cargo install --git https://github.com/HoshiyomiLusia/paneview.git --locked
```

## Usage

```bash
paneview
paneview --version
paneview check-update
paneview update
```

Keybindings:

| Key | Action |
| --- | --- |
| `Ctrl+Q` | Quit |
| `Ctrl+H/J/K/L` | Move focus |
| `Ctrl+\` | Vertical split |
| `Ctrl+-` | Horizontal split |
| `Ctrl+N` | New pane |
| `Ctrl+W` | Close focused pane |
| `Ctrl+S` | Toggle system panel |
| `Ctrl+C` | Send interrupt to focused pane |

Notes:

- Each pane starts the user's default shell.
- Unavailable system metrics are shown as `N/A`.
- PaneView is an MVP, not a full tmux replacement.
