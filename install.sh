#!/usr/bin/env sh
set -eu

REPO="HoshiyomiLusia/paneview"
BIN_NAME="paneview"
VERSION="${PANEVIEW_VERSION:-latest}"
INSTALL_DIR="${PANEVIEW_INSTALL_DIR:-}"

require_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "error: required command not found: $1" >&2
        exit 1
    fi
}

detect_target() {
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Darwin)
            os_part="apple-darwin"
            ;;
        Linux)
            os_part="unknown-linux-gnu"
            ;;
        *)
            echo "error: unsupported operating system: $os" >&2
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64 | amd64)
            arch_part="x86_64"
            ;;
        arm64 | aarch64)
            arch_part="aarch64"
            ;;
        *)
            echo "error: unsupported CPU architecture: $arch" >&2
            exit 1
            ;;
    esac

    target="${arch_part}-${os_part}"

    case "$target" in
        x86_64-unknown-linux-gnu | aarch64-unknown-linux-gnu | x86_64-apple-darwin | aarch64-apple-darwin)
            printf '%s\n' "$target"
            ;;
        *)
            echo "error: unsupported target: $target" >&2
            exit 1
            ;;
    esac
}

choose_install_dir() {
    if [ -n "$INSTALL_DIR" ]; then
        printf '%s\n' "$INSTALL_DIR"
        return
    fi

    if command -v "$BIN_NAME" >/dev/null 2>&1; then
        current_dir="$(dirname "$(command -v "$BIN_NAME")")"
        if [ -w "$current_dir" ]; then
            printf '%s\n' "$current_dir"
            return
        fi
    fi

    if [ -d /usr/local/bin ] && [ -w /usr/local/bin ]; then
        printf '%s\n' "/usr/local/bin"
        return
    fi

    printf '%s\n' "$HOME/.local/bin"
}

download_base_url() {
    target="$1"
    asset="paneview-${target}.tar.gz"

    if [ "$VERSION" = "latest" ]; then
        printf 'https://github.com/%s/releases/latest/download/%s\n' "$REPO" "$asset"
    else
        printf 'https://github.com/%s/releases/download/%s/%s\n' "$REPO" "$VERSION" "$asset"
    fi
}

verify_checksum() {
    archive="$1"
    checksum="$2"

    if [ ! -f "$checksum" ]; then
        echo "warning: checksum file was not downloaded; skipping verification" >&2
        return
    fi

    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum -c "$(basename "$checksum")"
        return
    fi

    if command -v shasum >/dev/null 2>&1; then
        expected="$(awk '{print $1}' "$checksum")"
        actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
        if [ "$expected" != "$actual" ]; then
            echo "error: checksum verification failed" >&2
            exit 1
        fi
        return
    fi

    echo "warning: neither sha256sum nor shasum is available; skipping verification" >&2
}

require_cmd uname
require_cmd curl
require_cmd tar
require_cmd mkdir
require_cmd chmod

target="$(detect_target)"
archive="paneview-${target}.tar.gz"
install_dir="$(choose_install_dir)"
url="$(download_base_url "$target")"

tmp_dir="$(mktemp -d 2>/dev/null || mktemp -d -t paneview)"
trap 'rm -rf "$tmp_dir"' EXIT INT TERM

echo "Downloading $url"
curl -fL "$url" -o "$tmp_dir/$archive"
curl -fL "$url.sha256" -o "$tmp_dir/$archive.sha256" || true

(
    cd "$tmp_dir"
    verify_checksum "$archive" "$archive.sha256"
    tar -xzf "$archive"
)

if [ ! -f "$tmp_dir/$BIN_NAME" ]; then
    echo "error: archive did not contain $BIN_NAME" >&2
    exit 1
fi

mkdir -p "$install_dir"
chmod 755 "$tmp_dir/$BIN_NAME"
cp "$tmp_dir/$BIN_NAME" "$install_dir/$BIN_NAME"

echo "Installed $BIN_NAME to $install_dir/$BIN_NAME"

case ":$PATH:" in
    *":$install_dir:"*) ;;
    *)
        echo "warning: $install_dir is not in PATH" >&2
        echo "add this to your shell profile:" >&2
        echo "  export PATH=\"$install_dir:\$PATH\"" >&2
        ;;
esac
