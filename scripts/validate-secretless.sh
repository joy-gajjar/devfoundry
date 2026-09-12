#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

command -v jq >/dev/null 2>&1 || {
    printf 'jq is required to validate JSON metadata\n' >&2
    exit 1
}

json_files=("$root"/release/*.json)
for file in "${json_files[@]}"; do
    jq empty "$file"
done

for script in "$root"/scripts/*.sh; do
    bash -n "$script"
done

version=$(awk -F'"' '/^version = / { print $2; exit }' "$root/Cargo.toml")
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]] || {
    printf 'invalid workspace version: %s\n' "$version" >&2
    exit 1
}

jq -e '.product == "devfoundry" and .credentials_required == false and .default_behavior == "disabled" and .automatic_download == false' \
    "$root/release/update-policy.json" >/dev/null
jq -e '.product == "devfoundry" and .private_keys_in_ci == false' \
    "$root/release/publication-plan.json" >/dev/null
jq -e '.product == "devfoundry" and ([.platforms[].status] | all(. == "supported"))' \
    "$root/release/platforms.json" >/dev/null

if grep -Eiq '^[[:space:]]*(env:.*(TOKEN|SECRET|KEY)|[A-Z0-9_]*(TOKEN|SECRET|PASSWORD|PRIVATE_KEY)[A-Z0-9_]*:)' "$root/.github/workflows"/*.yml; then
    printf 'workflow contains credential-like environment wiring\n' >&2
    exit 1
fi

for file in "$root/release"/package-managers.json "$root/release"/publication-plan.json "$root/release"/update-policy.json; do
    if grep -Eiq 'BEGIN (RSA|OPENSSH|EC|PGP) PRIVATE KEY|PRIVATE_KEY[=:]|(SECRET|TOKEN|PASSWORD)[=:][^<[:space:]]+' "$file"; then
        printf 'metadata contains secret material: %s\n' "$file" >&2
        exit 1
    fi
done

grep -F 'bash scripts/validate-secretless.sh' "$root/docs/security.md" >/dev/null
grep -F 'DEVFOUNDRY_NO_UPDATE_CHECK' "$root/docs/versioning-and-updates.md" >/dev/null
grep -F 'Signing Handoff' "$root/docs/release.md" >/dev/null
printf 'secretless shell, JSON, version, workflow, and documentation validation passed\n'
