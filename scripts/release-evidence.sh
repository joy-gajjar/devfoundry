#!/usr/bin/env bash
set -euo pipefail

usage() {
    printf 'usage: %s --archive PATH --checksum PATH --target TARGET --version VERSION --output PATH [--launch-doctor]\n' "$0" >&2
    exit 2
}

archive=''
checksum=''
target=''
version=''
output=''
launch_doctor=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --archive) archive=${2:-}; shift 2 ;;
        --checksum) checksum=${2:-}; shift 2 ;;
        --target) target=${2:-}; shift 2 ;;
        --version) version=${2:-}; shift 2 ;;
        --output) output=${2:-}; shift 2 ;;
        --launch-doctor) launch_doctor=true; shift ;;
        *) usage ;;
    esac
done

[[ -n "$archive" && -n "$checksum" && -n "$target" && -n "$version" && -n "$output" ]] || usage
[[ "$target" =~ ^[A-Za-z0-9._-]+$ && "$version" =~ ^[A-Za-z0-9._+-]+$ ]] || {
    printf 'target and version must be path-safe\n' >&2
    exit 1
}

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
verify_args=(--archive "$archive" --checksum "$checksum" --target "$target" --version "$version")
if "$launch_doctor"; then
    verify_args+=(--launch-doctor)
fi
"$script_dir/verify-release.sh" "${verify_args[@]}"

archive_name=$(basename -- "$archive")
checksum_name=$(basename -- "$checksum")
if command -v sha256sum >/dev/null 2>&1; then
    digest=$(sha256sum "$archive" | awk '{print $1}')
else
    digest=$(shasum -a 256 "$archive" | awk '{print $1}')
fi
mkdir -p "$(dirname -- "$output")"
cat >"$output" <<EOF
{
  "schema_version": 1,
  "product": "devfoundry",
  "target": "${target}",
  "version": "${version}",
  "archive": "${archive_name}",
  "checksum": "${checksum_name}",
  "sha256": "${digest}",
  "archive_verified": true,
  "doctor_smoke": ${launch_doctor},
  "signing": "external-trusted-host",
  "package_publication": "external-release-owner"
}
EOF
printf 'created release evidence %s\n' "$output"
