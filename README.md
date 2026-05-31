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

PaneView uses a **prefix key** (`Ctrl+B`, like tmux) so that shell shortcuts
such as `Ctrl+C`, `Ctrl+S`, `Ctrl+Q`, `Ctrl+W`, etc. reach the focused
shell unmodified. Press the prefix, release it, then press the command key.

| Chord | Action |
| --- | --- |
| `Ctrl+B  q` | Quit PaneView |
| `Ctrl+B  h / j / k / l` | Move pane focus (left / down / up / right) |
| `Ctrl+B  ← / ↓ / ↑ / →` | Same, with arrow keys |
| `Ctrl+B  |` *(or `\`)* | Split focused pane vertically |
| `Ctrl+B  -` | Split focused pane horizontally |
| `Ctrl+B  n` *(or `c`)* | Create a new pane |
| `Ctrl+B  x` *(or `w`)* | Close the focused pane |
| `Ctrl+B  s` | Toggle the system dashboard |
| `Ctrl+B  [` *(or `PageUp`)* | Enter scroll mode for the focused pane |

In scroll mode:

| Key | Action |
| --- | --- |
| `PageUp / PageDown` *(or `b / f`)* | Scroll one screen |
| `↑ / ↓` *(or `k / j`)* | Scroll one line |
| `g / Home`, `G / End` | Jump to the top / bottom of scrollback |
| `q / Esc` | Leave scroll mode |

The status bar's mode indicator reads `PREFIX` between the prefix and the
next key, and `SCROLL` while a pane is in scroll mode. The focused pane's
border turns yellow during scroll mode and the title shows `[scroll +N]`.

Notes:

- Each pane starts the user's default shell.
- ANSI colours, bold/italic/underline, and the cursor position are rendered
  inside each pane, so `ls --color`, `vim`, `htop`, etc. look correct.
- F1–F12, `Ctrl/Alt/Shift+arrows`, and bracketed paste are forwarded to
  the focused shell.
- Unavailable system metrics are shown as `N/A`.
- PaneView is not a tmux replacement.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or
  <https://opensource.org/licenses/MIT>)

at your option.
