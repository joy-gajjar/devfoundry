#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
binary="$work/devfoundry"
cat >"$binary" <<'EOF'
#!/bin/sh
if [ "${1:-}" = '--json' ] && [ "${2:-}" = 'doctor' ]; then
    printf '{"config_loaded":true,"provider_token_configured":false}\n'
fi
exit 0
EOF
chmod +x "$binary"
dist="$work/dist"

(cd "$root" && bash scripts/package-release.sh --binary "$binary" --target test-target --version 1.2.3 --output "$dist")
archive="$dist/devfoundry-1.2.3-test-target.tar.gz"
checksum="$archive.sha256"
(cd "$root" && bash scripts/verify-release.sh --archive "$archive" --checksum "$checksum" --target test-target --version 1.2.3)
(cd "$root" && bash scripts/release-evidence.sh --archive "$archive" --checksum "$checksum" --target test-target --version 1.2.3 --output "$dist/evidence.json" --launch-doctor)

printf 'placeholder signature\n' >"$dist/release.sig"
cat >"$dist/handoff.txt" <<EOF
archive=$(basename "$archive")
checksum=$(basename "$checksum")
signature=release.sig
public_key_fingerprint=0123456789ABCDEF
algorithm=sha256
verification_command=openssl dgst -sha256 -verify public-key.pem
workflow_run=12345
EOF
(cd "$root" && bash scripts/validate-release-handoff.sh --manifest "$dist/handoff.txt" --artifact-dir "$dist")
(cd "$root" && bash scripts/prepare-publication.sh --version 1.2.3 --artifact-dir "$dist" --output "$dist/publication.json")

prefix="$work/prefix"
(cd "$root" && bash scripts/manage-install.sh install "$archive" "$checksum" "$prefix")
(cd "$root" && bash scripts/package-release.sh --binary "$binary" --target test-target --version 1.2.4 --output "$dist")
(cd "$root" && bash scripts/manage-install.sh upgrade "$dist/devfoundry-1.2.4-test-target.tar.gz" "$dist/devfoundry-1.2.4-test-target.tar.gz.sha256" "$prefix")
[[ "$(readlink "$prefix/current")" == *1.2.4-test-target ]]
(cd "$root" && bash scripts/manage-install.sh rollback "$prefix")
[[ "$(readlink "$prefix/current")" == *1.2.3-test-target ]]
printf 'release automation tests passed\n'
