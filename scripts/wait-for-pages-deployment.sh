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

  # Success criteria (issue #479):
  #   1. deployment.json is served AND advertises "sha":"<expected_sha>".
  #      GitHub Pages deploys atomically -- the whole artifact (every HTML file
  #      plus this marker, all stamped by scripts/stamp-pages-artifact.sh in one
  #      step) flips live together. The marker SHA is therefore the AUTHORITATIVE
  #      freshness signal: if it reads the expected SHA, the index served beside
  #      it is the matching stamped build.
  #   2. The site root (index.html) is actually being served (HTTP 200).
  #   3. No un-stamped placeholders survive in the index -- a defensive net that
  #      catches a half-run or broken stamp step.
  # We deliberately do NOT require the raw SHA to appear in the index body. That
  # coupled the probe to every root page embedding the commit SHA verbatim, which
  # silently broke when the issue #479 landing page (/) shipped without cache-
  # busted asset refs: the marker had the right SHA but the index never did, so
  # this loop timed out for the full 300s and failed the whole pipeline (which in
  # turn gated the desktop release -> "Not available in latest release"). The
  # marker is sufficient; the index now carries ?v=<sha> refs too, but the probe
  # no longer depends on that.
  if [[ "$index_ok" == true && "$marker_ok" == true ]] &&
    grep -Eq "\"sha\"[[:space:]]*:[[:space:]]*\"${expected_sha}\"" "$marker_file" &&
    ! grep -Fq "__FORMAL_AI_ASSET_VERSION__" "$index_file" &&
    ! grep -Fq "__FORMAL_AI_VERSION__" "$index_file"; then
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
