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
        current_path="$(command -v "$BIN_NAME")"
        current_dir="$(dirname "$current_path")"
        if [ -w "$current_dir" ]; then
            printf '%s\n' "$current_dir"
            return
        fi

        echo "error: existing $BIN_NAME was found at $current_path, but $current_dir is not writable" >&2
        echo "rerun with the required permissions or set PANEVIEW_INSTALL_DIR explicitly" >&2
        exit 1
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
        echo "error: checksum file was not downloaded; refusing to install unverified binary" >&2
        exit 1
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

    echo "error: neither sha256sum nor shasum is available; cannot verify download" >&2
    exit 1
}

require_cmd uname
require_cmd curl
require_cmd tar
require_cmd mkdir
require_cmd chmod
require_cmd cp
require_cmd mv
require_cmd rm
require_cmd mktemp
require_cmd dirname

target="$(detect_target)"
archive="paneview-${target}.tar.gz"
install_dir="$(choose_install_dir)"
url="$(download_base_url "$target")"
staged_path=""

tmp_dir="$(mktemp -d 2>/dev/null || mktemp -d -t paneview)"
cleanup() {
    rm -rf "$tmp_dir"
    if [ -n "$staged_path" ] && [ -f "$staged_path" ]; then
        rm -f "$staged_path"
    fi
}
trap cleanup EXIT INT TERM

echo "Downloading $url"
curl -fL "$url" -o "$tmp_dir/$archive"
curl -fL "$url.sha256" -o "$tmp_dir/$archive.sha256"

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
if [ ! -w "$install_dir" ]; then
    echo "error: install directory is not writable: $install_dir" >&2
    exit 1
fi

chmod 755 "$tmp_dir/$BIN_NAME"

target_path="$install_dir/$BIN_NAME"
staged_path="$install_dir/.${BIN_NAME}.tmp.$$"
cp "$tmp_dir/$BIN_NAME" "$staged_path"
chmod 755 "$staged_path"
mv -f "$staged_path" "$target_path"

echo "Installed $BIN_NAME to $target_path"
if installed_version="$("$target_path" --version 2>/dev/null)"; then
    echo "$installed_version"
else
    echo "warning: installed binary could not be executed for version verification" >&2
fi

case ":$PATH:" in
    *":$install_dir:"*) ;;
    *)
        echo "warning: $install_dir is not in PATH" >&2
        echo "add this to your shell profile:" >&2
        echo "  export PATH=\"$install_dir:\$PATH\"" >&2
        ;;
esac
