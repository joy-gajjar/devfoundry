#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
package_metadata="$root/release/package-managers.json"
platform_metadata="$root/release/platforms.json"
update_policy="$root/release/update-policy.json"

command -v jq >/dev/null 2>&1 || {
    printf 'jq is required to validate distribution metadata\n' >&2
    exit 1
}

for file in "$package_metadata" "$platform_metadata" "$update_policy"; do
    jq empty "$file"
done

[[ "$(jq -r '.product' "$package_metadata")" == "devfoundry" ]]
[[ "$(jq -r '.product' "$platform_metadata")" == "devfoundry" ]]
[[ "$(jq -r '.product' "$update_policy")" == "devfoundry" ]]
[[ "$(jq -r '.credentials_required' "$update_policy")" == "false" ]]
[[ "$(jq -r '.default_behavior' "$update_policy")" == "disabled" ]]
[[ "$(jq -r '.notification.never_blocks_startup' "$update_policy")" == "true" ]]
[[ "$(jq -r '.automatic_download' "$update_policy")" == "false" ]]

for manager in homebrew cargo-binstall winget; do
    jq -e --arg manager "$manager" '.packages[] | select(.name == $manager)' "$package_metadata" >/dev/null
done

while IFS= read -r template; do
    [[ -f "$root/$template" ]] || { printf 'package template not found: %s\n' "$template" >&2; exit 1; }
done < <(jq -r '.packages[] | select(has("template")) | .template' "$package_metadata")

for template in "$root/release/homebrew/devfoundry.rb.template" "$root/release/winget/DevFoundry.DevFoundry.yaml.template"; do
    ! grep -Eiq 'PRIVATE_KEY|SECRET|TOKEN|PASSWORD|BEGIN .*PRIVATE KEY' "$template"
done

! grep -Eiq 'PRIVATE_KEY|SECRET|TOKEN|PASSWORD|BEGIN .*PRIVATE KEY' "$update_policy"

grep -F '<VERSION>' "$root/docs/release-notes-template.md" >/dev/null
grep -F 'DEVFOUNDRY_NO_UPDATE_CHECK' "$root/docs/versioning-and-updates.md" >/dev/null
printf 'distribution metadata and documentation validated\n'
