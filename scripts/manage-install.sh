#!/usr/bin/env bash
set -euo pipefail

usage() {
    printf 'usage: %s install|upgrade ARCHIVE CHECKSUM [PREFIX]\n' "$0" >&2
    printf '       %s rollback [PREFIX]\n' "$0" >&2
    exit 2
}

command=${1:-}
shift || true
prefix=${DEVFOUNDRY_PREFIX:-"$HOME/.local/share/devfoundry"}
current="$prefix/current"
previous="$prefix/previous"
versions="$prefix/versions"

case "$command" in
    install|upgrade)
        archive=${1:-}
        checksum=${2:-}
        [[ -n "$archive" && -n "$checksum" ]] || usage
        if [[ -n "${3:-}" ]]; then prefix=$3; current="$prefix/current"; previous="$prefix/previous"; versions="$prefix/versions"; fi
        [[ -f "$archive" && -f "$checksum" ]] || { printf 'archive and checksum are required\n' >&2; exit 1; }
        script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
        "$script_dir/verify-release.sh" --archive "$archive" --checksum "$checksum"
        name=$(tar -tzf "$archive" | awk -F/ 'NF > 1 { print $1; exit }')
        [[ -n "$name" ]] || { printf 'archive has no top-level release directory\n' >&2; exit 1; }
        mkdir -p "$versions"
        staging=$(mktemp -d "$versions/.install.XXXXXX")
        trap 'rm -rf "$staging"' EXIT
        tar -xzf "$archive" -C "$staging"
        [[ -x "$staging/$name/devfoundry" || -x "$staging/$name/devfoundry.exe" ]] || { printf 'release binary missing or not executable\n' >&2; exit 1; }
        version_dir="$versions/$name"
        rm -rf "$version_dir"
        mv "$staging/$name" "$version_dir"
        if [[ -L "$current" || -e "$current" ]]; then
            rm -f "$previous"
            mv "$current" "$previous"
        fi
        ln -s "$version_dir" "$current"
        printf '%s %s: %s\n' "$command" "$name" "$current"
        ;;
    rollback)
        if [[ -n "${1:-}" ]]; then prefix=$1; current="$prefix/current"; previous="$prefix/previous"; fi
        [[ -L "$previous" || -e "$previous" ]] || { printf 'no previous release is available\n' >&2; exit 1; }
        [[ -L "$current" || -e "$current" ]] || { printf 'current release is missing\n' >&2; exit 1; }
        old_current="$prefix/.current.rollback"
        rm -f "$old_current"
        mv "$current" "$old_current"
        mv "$previous" "$current"
        mv "$old_current" "$previous"
        printf 'rolled back %s\n' "$current"
        ;;
    *) usage ;;
esac
