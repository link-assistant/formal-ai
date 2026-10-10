#!/usr/bin/env bash
# Resolve which release tag the Desktop Release workflow should build assets for,
# and whether a build is needed at all.
#
# Extracted from .github/workflows/desktop-release.yml so the decision logic
# (the part that regressed in issue #479) is unit-testable with a mocked `gh`
# CLI. See rust/tests/unit/ci-cd/desktop_release_resolve.rs.
#
# ---------------------------------------------------------------------------
# Background (issue #479: "Not available in latest release" for all desktop apps)
# ---------------------------------------------------------------------------
# The automated CI/CD release (scripts/version-and-commit.rs) bumps the version
# in a NEW child commit ("chore: release vX.Y.Z"), annotates THAT commit with the
# vX.Y.Z tag, and creates the GitHub release from it -- all pushed with
# GITHUB_TOKEN. GitHub therefore:
#   * suppresses the `release` event for that release (recursion guard), and
#   * never starts a CI run for the child commit (also recursion guard).
# The "CI/CD Pipeline" run that DOES complete carries the PARENT commit's SHA
# (the commit CI actually ran on). The release tag points at the child commit,
# whose first parent IS that head SHA.
#
# The previous resolve logic required a tag whose commit EQUALS the workflow_run
# head SHA. Because the tag lives on the child commit, that match NEVER
# succeeded, so the build job was skipped, no assets were uploaded, and every
# /download entry read "Not available in latest release".
#
# ---------------------------------------------------------------------------
# Resolution tiers (workflow_run)
# ---------------------------------------------------------------------------
#   Tier 1 (defensive): a tag whose commit IS the head SHA. Future-proof in case
#           the release flow ever stops creating a child commit.
#   Tier 2 (normal):    the latest published release -- the auto-release child
#           commit whose first parent is the head SHA. A diagnostic check
#           requires that parent relationship, selecting another matching
#           published stable release if a newer tag is already latest.
# An idempotency / self-healing guard then skips the build only when the resolved
# release already carries every expected desktop asset. A partial release (for
# example macOS/Windows present but Linux missing) must rebuild so it self-heals.
#
# ---------------------------------------------------------------------------
# Inputs (environment)
# ---------------------------------------------------------------------------
#   EVENT                  github.event_name (release|workflow_dispatch|workflow_run)
#   INPUT_TAG              workflow_dispatch input tag (optional)
#   RELEASE_TAG            release event tag (github.event.release.tag_name)
#   REPO                   owner/name (required)
#   WORKFLOW_RUN_HEAD_SHA  head SHA of the completed CI run (workflow_run only)
#   GH_TOKEN               token for the gh CLI
#   GITHUB_OUTPUT          file to append `tag=`/`should_build=` (optional; the
#                          script also always prints the resolved values so local
#                          runs and tests can read them from stdout)
set -euo pipefail

EVENT="${EVENT:-}"
INPUT_TAG="${INPUT_TAG:-}"
RELEASE_TAG="${RELEASE_TAG:-}"
REPO="${REPO:?REPO is required}"
WORKFLOW_RUN_HEAD_SHA="${WORKFLOW_RUN_HEAD_SHA:-}"

tag=""
should_build=true
resolution="default"

group() { echo "::group::$*"; }
endgroup() { echo "::endgroup::"; }
log() { echo "[desktop-release-resolve] $*"; }

emit_outputs() {
  log "result: tag='${tag}' should_build='${should_build}' resolution='${resolution}'"
  # Issue #812: a healing build checks out the *tag*, so it packages that tag's
  # code -- not `main`. Run 29752745259 failed macOS signing on v0.300.0, which
  # predates the four packaging fixes already merged, and the red run read as
  # "desktop packaging is broken" when it meant "the last tag is old". Say so in
  # the run summary rather than leaving it to be re-diagnosed each time.
  if [ "$should_build" = "true" ] && [ -n "$tag" ]; then
    head_sha="$(gh api "repos/${REPO}/commits/HEAD" --jq .sha 2>/dev/null || echo "")"
    tag_sha="$(gh api "repos/${REPO}/commits/${tag}" --jq .sha 2>/dev/null || echo "")"
    if [ -n "$head_sha" ] && [ -n "$tag_sha" ] && [ "$head_sha" != "$tag_sha" ]; then
      echo "::notice title=Healing build of an existing tag::Packaging ${tag}, which is not the tip of the default branch. This run builds that tag's code; fixes merged after it are not included, so a failure here does not necessarily mean current code is broken."
    fi
  fi
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
      echo "tag=$tag"
      echo "should_build=$should_build"
    } >> "$GITHUB_OUTPUT"
  fi
}

# Select the same run's published release even when a newer tag is latest.
release_for_workflow_head() {
  local resolver
  resolver="$(cd "$(dirname "$0")" && pwd)/resolve-package-release.mjs"
  EVENT=workflow_run REPOSITORY="$REPO" RUN_HEAD="$WORKFLOW_RUN_HEAD_SHA" \
    RUN_ID="${WORKFLOW_RUN_ID:-}" RUN_ATTEMPT="${WORKFLOW_RUN_ATTEMPT:-}" \
    RUN_WORKFLOW_ID="${WORKFLOW_RUN_WORKFLOW_ID:-}" \
    RUN_BRANCH="${WORKFLOW_RUN_BRANCH:-}" RUN_REPOSITORY="${WORKFLOW_RUN_HEAD_REPOSITORY:-}" \
    RUN_CONCLUSION="${WORKFLOW_RUN_CONCLUSION:-}" \
    node "$resolver" | sed -n 's/^tag=//p'
}

latest_release_tag() {
  gh release view --repo "$REPO" --json tagName --jq .tagName 2>/dev/null || true
}

release_version_from_tag() {
  local value="$1"
  if [[ "$value" =~ (^|-)v?([0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)?) ]]; then
    printf '%s\n' "${BASH_REMATCH[2]}"
  fi
}

expected_desktop_assets() {
  local version="$1"
  printf '%s\n' \
    "formal-ai-desktop-macos-arm64-${version}.dmg" \
    "formal-ai-desktop-macos-arm64-${version}.zip" \
    "formal-ai-desktop-macos-x64-${version}.dmg" \
    "formal-ai-desktop-macos-x64-${version}.zip" \
    "formal-ai-desktop-windows-installer-x64-${version}.exe" \
    "formal-ai-desktop-windows-installer-arm64-${version}.exe" \
    "formal-ai-desktop-windows-portable-x64-${version}.exe" \
    "formal-ai-desktop-windows-portable-arm64-${version}.exe" \
    "formal-ai-desktop-linux-x64-${version}.AppImage" \
    "formal-ai-desktop-linux-arm64-${version}.AppImage" \
    "formal-ai-desktop-linux-x64-${version}.deb" \
    "formal-ai-desktop-linux-arm64-${version}.deb" \
    "formal-ai-desktop-linux-x64-${version}.tar.gz" \
    "formal-ai-desktop-linux-arm64-${version}.tar.gz" \
    "latest.yml" \
    "latest-mac.yml" \
    "latest-linux.yml"
  printf '%s\n' \
    "formal-ai-vscode-${version}.vsix" \
    "SHA256SUMS.txt" \
    "BUILD-PROVENANCE.txt"
  expected_cli_assets
  printf '%s\n' "formal-ai-native-source-${version}.json" "formal-ai-native-protocol-${version}.json"
  expected_native_targets | while IFS= read -r target; do printf '%s\n' "formal-ai-native-${target}-${version}.json"; done
  printf '%s\n' "formal-ai-signing-macos-arm64-${version}.json" "formal-ai-signing-macos-x64-${version}.json"
}

# Issue #1181: the `cli` job's archives belong to the same release, so a release
# that lacks one of them is partial too. The names come from the `cli` job's own
# matrix (`target:` and `archive:` of every `label: cli-*` leg), so a leg added
# there is required here without a second list.
DESKTOP_RELEASE_WORKFLOW="${DESKTOP_RELEASE_WORKFLOW:-$(dirname "$0")/../.github/workflows/desktop-release.yml}"
expected_cli_assets() {
  if [ ! -f "$DESKTOP_RELEASE_WORKFLOW" ]; then
    # Unknown matrix: require a name no release carries, so the guard builds
    # (the fail-safe direction) instead of silently dropping the CLI assets.
    log "warning: ${DESKTOP_RELEASE_WORKFLOW} not found; cannot list the CLI archives, building." >&2
    echo "formal-ai-cli-<matrix unknown>"
    return 0
  fi
  sed -n 's/.*target: *\([A-Za-z0-9_.-]*\), *label: *cli-[^,]*,.*archive: *\([a-z.]*\),.*/formal-ai-cli-\1.\2/p' \
    "$DESKTOP_RELEASE_WORKFLOW"
}

# Derive native target evidence from the same checked producer matrix.
expected_native_targets() {
  if [ ! -f "$DESKTOP_RELEASE_WORKFLOW" ]; then echo '<matrix unknown>'; return; fi
  sed -n '/^  native:/,/^  build:/p' "$DESKTOP_RELEASE_WORKFLOW" | sed -n 's/.*target: *\([A-Za-z0-9_.-]*\), *binext:.*/\1/p'
}

verify_durable_release() {
  local directory source_commit source_tree status
  directory="$(mktemp -d)"
  status=1
  if gh release download "$tag" --repo "$REPO" --dir "$directory" \
      --pattern 'formal-ai-native-*.json' --pattern 'formal-ai-signing-*.json' --pattern 'SHA256SUMS.txt' \
      && gh api "repos/$REPO/releases/tags/$tag" > "$directory/release-assets.json" \
      && source_commit="$(gh api "repos/$REPO/commits/$tag" --jq .sha)" \
      && source_tree="$(gh api "repos/$REPO/commits/$tag" --jq .commit.tree.sha)"; then
    expected_desktop_assets "$release_version" > "$directory/expected-assets.txt"
    if REPO="$REPO" node "$(dirname "$0")/native-release-evidence.mjs" published "$directory" "$release_version" "$source_commit" "$source_tree"; then status=0; fi
  fi
  rm -rf "$directory"
  return "$status"
}

group "desktop-release resolve inputs"
log "event                 = ${EVENT:-<none>}"
log "input_tag             = ${INPUT_TAG:-<none>}"
log "release_tag           = ${RELEASE_TAG:-<none>}"
log "repo                  = ${REPO}"
log "workflow_run_head_sha = ${WORKFLOW_RUN_HEAD_SHA:-<none>}"
endgroup

case "$EVENT" in
  release)
    tag="$RELEASE_TAG"
    resolution="release-event"
    ;;
  workflow_dispatch)
    tag="${INPUT_TAG:-}"
    resolution="workflow_dispatch-input"
    ;;
  workflow_run)
    if [ -z "$WORKFLOW_RUN_HEAD_SHA" ]; then
      should_build=false
      resolution="workflow_run-missing-head-sha"
      log "workflow_run payload carried no head SHA; skipping desktop build."
      emit_outputs
      exit 0
    fi

    # Authenticate the completed source before either published-tag tier.
    if ! EVENT=workflow_run REPOSITORY="$REPO" RUN_HEAD="$WORKFLOW_RUN_HEAD_SHA" \
      RUN_ID="${WORKFLOW_RUN_ID:-}" RUN_ATTEMPT="${WORKFLOW_RUN_ATTEMPT:-}" \
      RUN_WORKFLOW_ID="${WORKFLOW_RUN_WORKFLOW_ID:-}" \
      RUN_BRANCH="${WORKFLOW_RUN_BRANCH:-}" \
      RUN_REPOSITORY="${WORKFLOW_RUN_HEAD_REPOSITORY:-}" \
      RUN_CONCLUSION="${WORKFLOW_RUN_CONCLUSION:-}" \
      node --input-type=module - "$(dirname "$0")/resolve-package-release.mjs" <<'NODE'
import { pathToFileURL } from 'node:url';
const { qualifiesCompletedRun } = await import(pathToFileURL(process.argv[2]));
if (!qualifiesCompletedRun(process.env)) process.exit(1);
NODE
    then
      should_build=false
      resolution="workflow_run-unqualified-source"
      log "Completed workflow source could not be authenticated; skipping desktop build."
      emit_outputs
      exit 0
    fi

    # Tier 1 (defensive): a tag whose commit IS the completed CI head SHA.
    group "Tier 1: exact tag on head SHA ${WORKFLOW_RUN_HEAD_SHA}"
    exact="$(gh api "repos/$REPO/tags?per_page=100" --paginate \
      --jq ".[] | select(.commit.sha == \"$WORKFLOW_RUN_HEAD_SHA\") | .name" 2>/dev/null \
      | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+' | head -n 1 || true)"
    log "exact-match tag: ${exact:-<none>}"
    endgroup

    if [ -n "$exact" ]; then
      tag="$exact"
      resolution="workflow_run-exact-sha"
    else
      # Tier 2 (normal): the auto-release tags a CHILD "chore: release vX.Y.Z"
      # commit whose first parent is this head SHA, so no tag points at the head
      # SHA directly. Resolve the latest published release -- that child release.
      group "Tier 2: latest published release (auto-release child commit)"
      tag="$(latest_release_tag)"
      log "latest release tag: ${tag:-<none>}"
      if [ -n "$tag" ]; then
        # Require the latest release to descend from this completed CI run.
        # `gh api .../commits/<tag>` dereferences the annotated tag to its commit.
        parent="$(gh api "repos/$REPO/commits/$tag" --jq '.parents[0].sha' 2>/dev/null || true)"
        if [ -n "$parent" ] && [ "$parent" = "$WORKFLOW_RUN_HEAD_SHA" ]; then
          log "confirmed: ${tag} commit parent is the CI head SHA (auto-release child)."
          resolution="workflow_run-child-of-head"
        else
          log "latest release does not belong to this run; selecting its matching published stable release."
          tag="$(release_for_workflow_head)"
          resolution="workflow_run-matched-published-release"
        fi
      fi
      endgroup
    fi

    if [ -z "$tag" ]; then
      should_build=false
      resolution="workflow_run-no-release"
      log "No published release found; skipping desktop build."
      emit_outputs
      exit 0
    fi

    if ! gh release view "$tag" --repo "$REPO" --json tagName >/dev/null 2>&1; then
      should_build=false
      resolution="workflow_run-release-missing"
      log "No GitHub release exists for resolved tag ${tag}; skipping desktop build."
      emit_outputs
      exit 0
    fi
    ;;
esac

# release / workflow_dispatch with an empty tag fall back to the latest release.
if [ -z "$tag" ]; then
  tag="$(latest_release_tag)"
  resolution="${resolution}+latest"
fi

if [ -z "$tag" ]; then
  log "Could not resolve any release tag to build; skipping." >&2
  should_build=false
  emit_outputs
  exit 0
fi

log "Resolved release tag: ${tag}"

# Idempotency / self-healing guard for automatic (workflow_run) builds: only
# skip when the resolved release already has the complete expected desktop
# artifact set. This:
#   * avoids rebuilding complete assets that already exist (pipeline re-runs, or
#     runs that did not cut a new release and fall back to the latest one),
#   * self-heals the original empty-asset backlog, and
#   * self-heals partial releases such as v0.204.0, where macOS/Windows assets
#     existed but Linux artifacts were still missing.
# Manual `release`/`workflow_dispatch` runs intentionally rebuild (clobber) so a
# maintainer can force a refresh.
group "Idempotency guard: required desktop assets on ${tag}"
release_version="$(release_version_from_tag "$tag" || true)"
if [ -z "$release_version" ]; then
  log "Could not parse semver from ${tag}; leaving build enabled rather than risking a silent skip."
else
  # Keep the query's exit status: a failed `gh` and a release with genuinely no
  # assets both yield an empty list, but only one of them is a problem worth
  # naming in the log. Either way the fail-safe direction is the same (build),
  # so the status only drives diagnostics, never the decision.
  if existing_names="$(gh release view "$tag" --repo "$REPO" --json assets \
    --jq '.assets[].name | select(startswith("formal-ai-desktop-") or startswith("formal-ai-cli-") or startswith("formal-ai-vscode-") or startswith("formal-ai-native-") or startswith("formal-ai-signing-") or . == "latest.yml" or . == "latest-mac.yml" or . == "latest-linux.yml" or . == "SHA256SUMS.txt" or . == "BUILD-PROVENANCE.txt")' 2>/dev/null)"; then
    :
  else
    log "warning: could not list assets for ${tag} (gh exited non-zero); treating them as absent and building."
    existing_names=""
  fi
  existing_count="$(printf '%s\n' "$existing_names" | sed '/^$/d' | wc -l | tr -d ' ')"
  missing=()
  while IFS= read -r expected; do
    # Feed grep directly: with pipefail, `printf | grep -q` can report failure
    # after a successful early match when printf receives SIGPIPE.
    if ! grep -Fxq "$expected" <<<"$existing_names"; then
      missing+=("$expected")
    fi
  done < <(expected_desktop_assets "$release_version")

  log "release version: ${release_version}"
  log "existing desktop assets: ${existing_count}"
  log "required release assets: desktop/updater, versioned VSIX, manifests, CLI archives and source/protocol/native/signing observations"
  if [ ${#missing[@]} -eq 0 ]; then
    log "all required desktop assets are present."
  else
    log "missing required desktop assets (${#missing[@]}):"
    printf '  %s\n' "${missing[@]}"
  fi
fi
endgroup
if [ "$EVENT" = "workflow_run" ] && [ -n "${release_version:-}" ] && [ ${#missing[@]} -eq 0 ]; then
  if verify_durable_release; then
    should_build=false
    resolution="${resolution}+already-has-all-assets"
    log "Release ${tag} has complete byte-consistent workflow-authenticated evidence; skipping automatic build."
  else
    log "Durable release evidence did not verify; rebuilding rather than accepting asset names alone."
  fi
fi

emit_outputs
