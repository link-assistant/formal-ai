#!/usr/bin/env bash
set -euo pipefail

artifact_dir="${1:-src/web}"
asset_version="${2:-${FORMAL_AI_ASSET_VERSION:-}}"
expected_sha="${3:-${GITHUB_SHA:-$asset_version}}"
formal_ai_version="${4:-${FORMAL_AI_VERSION:-}}"

if [[ -z "$asset_version" ]]; then
  echo "::error::asset version is required"
  exit 1
fi

if [[ -z "$expected_sha" ]]; then
  echo "::error::expected deployment SHA is required"
  exit 1
fi

# Derive the formal-ai release version from Cargo.toml when it is not passed
# explicitly. Issue #72: without this step `src/web/index.html` keeps the
# `__FORMAL_AI_VERSION__` placeholder (or the historical hardcoded `0.16.0`),
# so the deployed GitHub Pages site advertises a stale version and every
# issue report quotes the wrong number.
if [[ -z "$formal_ai_version" ]]; then
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  cargo_toml="${script_dir}/../Cargo.toml"
  if [[ -f "$cargo_toml" ]]; then
    formal_ai_version="$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$cargo_toml" | head -n 1)"
  fi
fi

if [[ -z "$formal_ai_version" ]]; then
  echo "::error::formal-ai version is required (pass as 4th arg, set FORMAL_AI_VERSION, or ensure Cargo.toml is present)"
  exit 1
fi

index_html="${artifact_dir}/index.html"
if [[ ! -f "$index_html" ]]; then
  echo "::error file=${index_html}::GitHub Pages artifact is missing index.html"
  exit 1
fi

escaped_version="$(printf '%s' "$asset_version" | sed 's/[\/&]/\\&/g')"
sed -i "s/__FORMAL_AI_ASSET_VERSION__/${escaped_version}/g" "$index_html"

if grep -q "__FORMAL_AI_ASSET_VERSION__" "$index_html"; then
  echo "::error file=${index_html}::Failed to replace the Pages asset version placeholder"
  exit 1
fi

escaped_formal_ai_version="$(printf '%s' "$formal_ai_version" | sed 's/[\/&]/\\&/g')"
sed -i "s/__FORMAL_AI_VERSION__/${escaped_formal_ai_version}/g" "$index_html"

if grep -q "__FORMAL_AI_VERSION__" "$index_html"; then
  echo "::error file=${index_html}::Failed to replace the formal-ai version placeholder"
  exit 1
fi

# Sanity check: index.html must advertise the freshly stamped formal-ai
# version so issue reports include the correct release. Failing the deploy
# here is cheaper than discovering a stale meta tag on the live site.
if ! grep -q "<meta name=\"formal-ai-version\" content=\"${formal_ai_version}\"" "$index_html"; then
  echo "::error file=${index_html}::Stamped index.html does not advertise formal-ai version ${formal_ai_version}"
  exit 1
fi

cat > "${artifact_dir}/deployment.json" <<EOF
{
  "sha": "${expected_sha}",
  "asset_version": "${asset_version}",
  "formal_ai_version": "${formal_ai_version}",
  "run_id": "${GITHUB_RUN_ID:-}",
  "run_attempt": "${GITHUB_RUN_ATTEMPT:-}"
}
EOF

echo "Stamped GitHub Pages artifact with SHA ${expected_sha} and formal-ai version ${formal_ai_version}"
