#!/usr/bin/env bash
set -euo pipefail

usage() {
    printf 'usage: %s --archive PATH --checksum PATH [--target TARGET] [--version VERSION] [--launch-doctor]\n' "$0" >&2
    exit 2
}

archive=''
checksum=''
target=''
version=''
launch_doctor=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --archive) archive=${2:-}; shift 2 ;;
        --checksum) checksum=${2:-}; shift 2 ;;
        --target) target=${2:-}; shift 2 ;;
        --version) version=${2:-}; shift 2 ;;
        --launch-doctor) launch_doctor=true; shift ;;
        *) usage ;;
    esac
done

[[ -n "$archive" && -n "$checksum" ]] || usage
[[ -f "$archive" ]] || { printf 'archive not found: %s\n' "$archive" >&2; exit 1; }
[[ -f "$checksum" ]] || { printf 'checksum not found: %s\n' "$checksum" >&2; exit 1; }

archive_dir=$(CDPATH= cd -- "$(dirname -- "$archive")" && pwd)
checksum_path=$(CDPATH= cd -- "$(dirname -- "$checksum")" && pwd)/$(basename -- "$checksum")

if command -v sha256sum >/dev/null 2>&1; then
    (cd "$archive_dir" && sha256sum -c "$checksum_path")
else
    (cd "$archive_dir" && shasum -a 256 -c "$checksum_path")
fi

listing=$(tar -tzf "$archive")
if [[ -n "$target" && -n "$version" ]]; then
    name="devfoundry-${version}-${target}"
    expected_binary='devfoundry'
    [[ "$target" == *windows* ]] && expected_binary='devfoundry.exe'
    printf '%s\n' "$listing" | grep -Fx "${name}/${expected_binary}" >/dev/null
    printf '%s\n' "$listing" | grep -Fx "${name}/LICENSE" >/dev/null
else
    name=$(printf '%s\n' "$listing" | awk -F/ 'NF == 2 && $2 ~ /^devfoundry(\.exe)?$/ { print $1; exit }')
    [[ -n "$name" ]] || { printf 'archive binary is missing\n' >&2; exit 1; }
    printf '%s\n' "$listing" | grep -Fx "${name}/LICENSE" >/dev/null
    expected_binary=$(printf '%s\n' "$listing" | awk -F/ -v root="$name" '$1 == root && $2 ~ /^devfoundry(\.exe)?$/ { print $2; exit }')
fi

extract_dir=$(mktemp -d)
trap 'rm -rf "$extract_dir"' EXIT
tar -xzf "$archive" -C "$extract_dir"
binary_path="$extract_dir/$name/$expected_binary"
[[ -x "$binary_path" ]] || { printf 'release binary is not executable: %s\n' "$expected_binary" >&2; exit 1; }
if "$launch_doctor"; then
    (cd "$extract_dir" && env -u GITHUB_COPILOT_TOKEN "$binary_path" --json doctor >"$extract_dir/doctor.json")
    grep -F '"config_loaded":true' "$extract_dir/doctor.json" >/dev/null
    grep -F '"provider_token_configured":false' "$extract_dir/doctor.json" >/dev/null
fi
printf 'verified %s\n' "$archive"
