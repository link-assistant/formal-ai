#!/usr/bin/env bash
set -euo pipefail

artifact_dir="${1:-src/web}"
asset_version="${2:-${FORMAL_AI_ASSET_VERSION:-}}"
expected_sha="${3:-${GITHUB_SHA:-$asset_version}}"

if [[ -z "$asset_version" ]]; then
  echo "::error::asset version is required"
  exit 1
fi

if [[ -z "$expected_sha" ]]; then
  echo "::error::expected deployment SHA is required"
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

cat > "${artifact_dir}/deployment.json" <<EOF
{
  "sha": "${expected_sha}",
  "asset_version": "${asset_version}",
  "run_id": "${GITHUB_RUN_ID:-}",
  "run_attempt": "${GITHUB_RUN_ATTEMPT:-}"
}
EOF

echo "Stamped GitHub Pages artifact with SHA ${expected_sha}"
