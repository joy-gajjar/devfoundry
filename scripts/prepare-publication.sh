#!/usr/bin/env bash
set -euo pipefail

usage() {
    printf 'usage: %s --version VERSION --artifact-dir DIR --output PATH\n' "$0" >&2
    exit 2
}

version=''
artifact_dir=''
output=''
while [[ $# -gt 0 ]]; do
    case "$1" in
        --version) version=${2:-}; shift 2 ;;
        --artifact-dir) artifact_dir=${2:-}; shift 2 ;;
        --output) output=${2:-}; shift 2 ;;
        *) usage ;;
    esac
done

[[ -n "$version" && -n "$artifact_dir" && -n "$output" ]] || usage
[[ "$version" =~ ^[A-Za-z0-9._+-]+$ ]] || { printf 'version must be path-safe\n' >&2; exit 1; }
[[ -d "$artifact_dir" ]] || { printf 'artifact directory not found: %s\n' "$artifact_dir" >&2; exit 1; }

archive_count=$(find "$artifact_dir" -type f -name '*.tar.gz' -print | wc -l | tr -d ' ')
checksum_count=$(find "$artifact_dir" -type f -name '*.sha256' -print | wc -l | tr -d ' ')
[[ "$archive_count" -gt 0 && "$archive_count" == "$checksum_count" ]] || {
    printf 'publication handoff requires matching archives and checksums\n' >&2
    exit 1
}

mkdir -p "$(dirname -- "$output")"
cat >"$output" <<EOF
{
  "schema_version": 1,
  "product": "devfoundry",
  "version": "${version}",
  "mode": "handoff-only",
  "artifact_directory": "$(basename -- "$artifact_dir")",
  "archives": ${archive_count},
  "checksums": ${checksum_count},
  "signing": "external-trusted-host",
  "published": false,
  "private_keys_in_ci": false
}
EOF
printf 'created publication handoff %s\n' "$output"
