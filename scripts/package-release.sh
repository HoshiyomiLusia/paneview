#!/usr/bin/env bash
set -euo pipefail

target="${1:?usage: scripts/package-release.sh <rust-target>}"
binary="paneview"
archive="paneview-${target}.tar.gz"
dist_dir="dist"
staging_dir="$(mktemp -d)"

cleanup() {
    rm -rf "$staging_dir"
}
trap cleanup EXIT

cargo build --release --locked --target "$target"

mkdir -p "$dist_dir"
cp "target/${target}/release/${binary}" "$staging_dir/${binary}"
cp README.md "$staging_dir/README.md"
chmod 755 "$staging_dir/${binary}"

tar -czf "${dist_dir}/${archive}" -C "$staging_dir" "$binary" README.md

(
    cd "$dist_dir"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$archive" >"${archive}.sha256"
    else
        shasum -a 256 "$archive" >"${archive}.sha256"
    fi
)

echo "Created ${dist_dir}/${archive}"
