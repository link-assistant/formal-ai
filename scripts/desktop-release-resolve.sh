#!/usr/bin/env bash
# Resolve which release tag the Desktop Release workflow should build assets for,
# and whether a build is needed at all.
#
# Extracted from .github/workflows/desktop-release.yml so the decision logic
# (the part that regressed in issue #479) is unit-testable with a mocked `gh`
# CLI. See tests/unit/ci-cd/desktop_release_resolve.rs.
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
#           confirms the parent relationship and records it in the log, but the
#           build proceeds regardless so the page self-heals.
# An idempotency / self-healing guard then skips the build only when the resolved
# release already carries desktop assets.
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
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
      echo "tag=$tag"
      echo "should_build=$should_build"
    } >> "$GITHUB_OUTPUT"
  fi
}

latest_release_tag() {
  gh release view --repo "$REPO" --json tagName --jq .tagName 2>/dev/null || true
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
        # Diagnostic only: confirm the latest release descends from this CI run.
        # `gh api .../commits/<tag>` dereferences the annotated tag to its commit.
        parent="$(gh api "repos/$REPO/commits/$tag" --jq '.parents[0].sha' 2>/dev/null || true)"
        if [ -n "$parent" ] && [ "$parent" = "$WORKFLOW_RUN_HEAD_SHA" ]; then
          log "confirmed: ${tag} commit parent is the CI head SHA (auto-release child)."
          resolution="workflow_run-child-of-head"
        else
          log "note: ${tag} commit parent='${parent:-<none>}' != head SHA; using latest release as self-healing fallback."
          resolution="workflow_run-latest-fallback"
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
# build when the resolved release is missing desktop assets. This:
#   * avoids rebuilding assets that already exist (pipeline re-runs, or runs that
#     did not cut a new release and fall back to the latest one), and
#   * self-heals the backlog this bug created -- the first pipeline to complete
#     after this fix lands sees the asset-less latest release and builds it.
# Manual `release`/`workflow_dispatch` runs intentionally rebuild (clobber) so a
# maintainer can force a refresh.
group "Idempotency guard: existing desktop assets on ${tag}"
existing="$(gh release view "$tag" --repo "$REPO" --json assets \
  --jq '[.assets[].name | select(startswith("formal-ai-desktop-"))] | length' 2>/dev/null || echo 0)"
existing="${existing//[^0-9]/}"
existing="${existing:-0}"
log "existing desktop assets: ${existing}"
endgroup
if [ "$EVENT" = "workflow_run" ] && [ "$existing" -gt 0 ]; then
  should_build=false
  resolution="${resolution}+already-has-assets"
  log "Release ${tag} already has ${existing} desktop assets; skipping automatic build."
fi

emit_outputs
