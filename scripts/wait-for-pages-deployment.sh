#!/usr/bin/env bash
set -euo pipefail

pages_url="${1:-}"
expected_sha="${2:-}"
timeout_seconds="${3:-300}"
interval_seconds="${4:-5}"

if [[ -z "$pages_url" || -z "$expected_sha" ]]; then
  echo "Usage: $0 <pages-url> <expected-sha> [timeout-seconds] [interval-seconds]"
  exit 2
fi

base_url="${pages_url%/}"
deadline=$((SECONDS + timeout_seconds))
attempt=1
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

while true; do
  cache_buster="deployment_sha=${expected_sha}&attempt=${attempt}&t=$(date +%s)"
  index_file="${tmp_dir}/index.html"
  marker_file="${tmp_dir}/deployment.json"
  index_url="${base_url}/?${cache_buster}"
  marker_url="${base_url}/deployment.json?${cache_buster}"

  index_ok=false
  marker_ok=false
  if curl -fsSL --retry 2 --retry-delay 1 -o "$index_file" "$index_url"; then
    index_ok=true
  fi
  if curl -fsSL --retry 2 --retry-delay 1 -o "$marker_file" "$marker_url"; then
    marker_ok=true
  fi

  if [[ "$index_ok" == true && "$marker_ok" == true ]] &&
    grep -Eq "\"sha\"[[:space:]]*:[[:space:]]*\"${expected_sha}\"" "$marker_file" &&
    grep -Fq "$expected_sha" "$index_file" &&
    ! grep -Fq "__FORMAL_AI_ASSET_VERSION__" "$index_file"; then
    echo "GitHub Pages is serving deployment ${expected_sha}"
    exit 0
  fi

  if ((SECONDS >= deadline)); then
    echo "::error::Timed out waiting for GitHub Pages to serve deployment ${expected_sha}"
    echo "::group::Last deployment marker"
    if [[ -s "$marker_file" ]]; then
      sed -n '1,80p' "$marker_file"
    else
      echo "No deployment marker fetched from ${marker_url}"
    fi
    echo "::endgroup::"
    echo "::group::Last index head"
    if [[ -s "$index_file" ]]; then
      sed -n '1,80p' "$index_file"
    else
      echo "No index fetched from ${index_url}"
    fi
    echo "::endgroup::"
    exit 1
  fi

  echo "Waiting for GitHub Pages deployment ${expected_sha} (attempt ${attempt})"
  sleep "$interval_seconds"
  attempt=$((attempt + 1))
done
