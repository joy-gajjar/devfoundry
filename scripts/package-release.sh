#!/usr/bin/env bash
set -euo pipefail

usage() {
    printf 'usage: %s --binary PATH --target TARGET --version VERSION --output DIR\n' "$0" >&2
    exit 2
}

binary=''
target=''
version=''
output=''
while [[ $# -gt 0 ]]; do
    case "$1" in
        --binary) binary=${2:-}; shift 2 ;;
        --target) target=${2:-}; shift 2 ;;
        --version) version=${2:-}; shift 2 ;;
        --output) output=${2:-}; shift 2 ;;
        *) usage ;;
    esac
done

[[ -n "$binary" && -n "$target" && -n "$version" && -n "$output" ]] || usage
[[ -f "$binary" ]] || { printf 'binary not found: %s\n' "$binary" >&2; exit 1; }
[[ -f LICENSE ]] || { printf 'LICENSE is required\n' >&2; exit 1; }
[[ "$version" != */* && "$version" != .* ]] || { printf 'version must be a path-safe value\n' >&2; exit 1; }
[[ "$target" != */* && "$target" != .* ]] || { printf 'target must be a path-safe value\n' >&2; exit 1; }

name="devfoundry-${version}-${target}"
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
mkdir -p "$stage/$name"
cp "$binary" "$stage/$name/devfoundry$( [[ "$target" == *windows* ]] && printf '.exe' )"
cp LICENSE "$stage/$name/LICENSE"
cp README.md "$stage/$name/README.md" 2>/dev/null || true

mkdir -p "$output"
archive="$output/$name.tar.gz"
tar -czf "$archive" -C "$stage" "$name"

if command -v sha256sum >/dev/null 2>&1; then
    (cd "$output" && sha256sum "$(basename "$archive")" > "$(basename "$archive").sha256")
elif command -v shasum >/dev/null 2>&1; then
    (cd "$output" && shasum -a 256 "$(basename "$archive")" > "$(basename "$archive").sha256")
else
    printf 'no SHA-256 tool available\n' >&2
    exit 1
fi

tar -tzf "$archive" | grep -F "${name}/devfoundry" >/dev/null
tar -tzf "$archive" | grep -F "${name}/LICENSE" >/dev/null
if command -v sha256sum >/dev/null 2>&1; then
    (cd "$output" && sha256sum -c "$(basename "$archive").sha256" >/dev/null)
else
    (cd "$output" && shasum -a 256 -c "$(basename "$archive").sha256" >/dev/null)
fi
printf 'created %s\n' "$archive"
printf 'created %s\n' "$archive.sha256"
