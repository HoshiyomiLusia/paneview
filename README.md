# PaneView

## Project Overview

PaneView is a terminal UI for running PTY-backed split shell panes while watching local system and network status on macOS and Linux.

![PaneView terminal preview](docs/demo-preview.svg)

It provides:

- Real shell panes backed by PTYs.
- Vertical and horizontal pane splits.
- CPU, memory, disk, network, interface, host, kernel, and uptime views.
- A release installer, update check, and self-update command.

## Environment Dependencies

PaneView is distributed as prebuilt release binaries. Normal users do not need Rust, Cargo, Git, a compiler, or a source checkout.

The installer only needs a network downloader, archive extraction, and valid CA certificates.

### macOS

Supported:

- Apple Silicon: `aarch64-apple-darwin`
- Intel: `x86_64-apple-darwin`

Check the required tools only if installation fails:

```bash
command -v curl
command -v tar
```

`curl` and `tar` are included with normal macOS installations. If either command is missing, install the Apple command line tools or use another downloader and extract the matching archive from GitHub Releases.

### Linux

Supported:

- ARM64: `aarch64-unknown-linux-gnu`
- x86_64: `x86_64-unknown-linux-gnu`

Check the required tools only if installation fails:

```bash
command -v curl
command -v tar
```

Install missing runtime tools with your distribution package manager:

```bash
# Debian / Ubuntu
sudo apt-get update
sudo apt-get install -y curl tar ca-certificates
```

```bash
# Fedora
sudo dnf install curl tar ca-certificates
```

```bash
# Arch Linux
sudo pacman -S curl tar ca-certificates
```

### Windows

Windows is not supported.

## Project Installation

Install the latest release:

```bash
curl -fsSL https://raw.githubusercontent.com/HoshiyomiLusia/paneview/main/install.sh | sh
```

The installer downloads the matching binary archive for the current platform and installs `paneview` into the first writable location it can use.

Install to a custom directory:

```bash
PANEVIEW_INSTALL_DIR="$HOME/bin" sh -c 'curl -fsSL https://raw.githubusercontent.com/HoshiyomiLusia/paneview/main/install.sh | sh'
```

Install a specific release:

```bash
PANEVIEW_VERSION="v0.1.7" sh -c 'curl -fsSL https://raw.githubusercontent.com/HoshiyomiLusia/paneview/main/install.sh | sh'
```

## Usage

Run the TUI:

```bash
paneview
```

Other commands:

```bash
paneview --version
paneview check-update
paneview update
```

Keybindings:

| Key | Action |
| --- | --- |
| `Ctrl+Q` | Quit PaneView |
| `Ctrl+H/J/K/L` | Move pane focus |
| `Ctrl+\` | Split focused pane vertically |
| `Ctrl+-` | Split focused pane horizontally |
| `Ctrl+N` | Create a new pane |
| `Ctrl+W` | Close the focused pane |
| `Ctrl+S` | Toggle the system panel |
| `Ctrl+C` | Send interrupt to the focused pane process |

Notes:

- Each pane starts the user's default shell.
- Unavailable system metrics are shown as `N/A`.
- PaneView is not a tmux replacement.
