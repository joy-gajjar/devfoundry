#!/usr/bin/env bash
set -euo pipefail

usage() {
    printf 'usage: %s --manifest PATH --artifact-dir DIR\n' "$0" >&2
    exit 2
}

manifest=''
artifact_dir=''
while [[ $# -gt 0 ]]; do
    case "$1" in
        --manifest) manifest=${2:-}; shift 2 ;;
        --artifact-dir) artifact_dir=${2:-}; shift 2 ;;
        *) usage ;;
    esac
done

[[ -f "$manifest" && -d "$artifact_dir" ]] || usage

value() {
    awk -F= -v key="$1" '$1 == key { print substr($0, index($0, "=") + 1); found=1 } END { if (!found) exit 1 }' "$manifest"
}

archive=$(value archive)
checksum=$(value checksum)
signature=$(value signature)
fingerprint=$(value public_key_fingerprint)
algorithm=$(value algorithm)
verification_command=$(value verification_command)
workflow_run=$(value workflow_run)

for item in "$archive" "$checksum" "$signature"; do
    [[ "$item" != */* && "$item" != .* ]] || { printf 'handoff artifact names must be basenames\n' >&2; exit 1; }
    [[ -f "$artifact_dir/$item" ]] || { printf 'handoff artifact not found: %s\n' "$item" >&2; exit 1; }
done
[[ "$algorithm" == "sha256" || "$algorithm" == "sha256with-ed25519" || "$algorithm" == "sha256with-minisign" ]] || {
    printf 'unsupported handoff algorithm: %s\n' "$algorithm" >&2
    exit 1
}
[[ "$fingerprint" =~ ^[A-Fa-f0-9:]{16,}$ ]] || { printf 'invalid public key fingerprint\n' >&2; exit 1; }
[[ "$workflow_run" =~ ^[0-9]+$ ]] || { printf 'workflow_run must be numeric\n' >&2; exit 1; }
[[ -n "$verification_command" ]] || { printf 'verification_command is required\n' >&2; exit 1; }
if grep -Eiq 'BEGIN (RSA|OPENSSH|EC|PGP) PRIVATE KEY|PRIVATE_KEY|SECRET|TOKEN|PASSWORD' "$manifest"; then
    printf 'signing handoff contains private-key or secret material\n' >&2
    exit 1
fi

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
"$script_dir/verify-release.sh" \
    --archive "$artifact_dir/$archive" \
    --checksum "$artifact_dir/$checksum"
printf 'validated signing handoff for %s (workflow %s)\n' "$archive" "$workflow_run"
