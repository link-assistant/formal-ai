#!/usr/bin/env bash
# Fail the release when the image we just pushed to GHCR is not anonymously
# pullable.
#
# Issue #1001: a container package created by a `GITHUB_TOKEN` push starts
# *private*, and it does not inherit the visibility of the repository it is
# linked to -- only its access permissions. Nothing in the release pipeline
# ever looked at the result, so every published image since the first push was
# private and no job failed. Downstream consumers discovered it months later as
# `docker: Error response from daemon: error from registry: unauthorized`.
#
# `docker pull`'s "unauthorized" is ambiguous -- a typo in the image name says
# the same word -- so this probes GHCR's anonymous token endpoint instead,
# which distinguishes the two cases:
#
#   200 + token   the package is public; an anonymous pull works
#   401           the package exists but is PRIVATE
#   403           no such package, or invisible to an anonymous caller
#                 (i.e. the push did not happen)
#
# No credentials are used or needed: sending an Authorization header here would
# turn a private package into a 200 and recreate the false negative. The token
# in a 200 response is never printed.
#
# INPUT (environment)
#   GHCR_IMAGE                       image reference, e.g.
#                                    "ghcr.io/owner/name" (a ":tag" or "@digest"
#                                    suffix is ignored -- visibility is per
#                                    package, not per tag).
#   GHCR_TOKEN_ENDPOINT              token endpoint; override for tests.
#                                    Default: https://ghcr.io/token
#   VERIFY_GHCR_VISIBILITY_RETRIES   attempts before giving up on a transport
#                                    error or a 5xx. Default: 3
#   VERIFY_GHCR_VISIBILITY_DELAY     seconds between attempts. Default: 5
#   VERIFY_GHCR_VISIBILITY_VERBOSE   set to 1 to trace each attempt on stderr.
#                                    Off by default.
#
# Exit status: 0 when the image is public, 1 otherwise.
set -euo pipefail

GHCR_IMAGE="${GHCR_IMAGE:-}"
GHCR_TOKEN_ENDPOINT="${GHCR_TOKEN_ENDPOINT:-https://ghcr.io/token}"
retries="${VERIFY_GHCR_VISIBILITY_RETRIES:-3}"
delay="${VERIFY_GHCR_VISIBILITY_DELAY:-5}"

trace() {
  if [ "${VERIFY_GHCR_VISIBILITY_VERBOSE:-}" = "1" ]; then
    echo "verify-ghcr-visibility: $*" >&2
  fi
}

if [ -z "$GHCR_IMAGE" ]; then
  echo "::error::GHCR_IMAGE is not set; nothing to verify"
  exit 1
fi

# ghcr.io/owner/name:tag -> owner/name
repository="${GHCR_IMAGE#ghcr.io/}"
repository="${repository%%@*}"
repository="${repository%%:*}"

if [ "$repository" = "$GHCR_IMAGE" ] && [ "${GHCR_IMAGE#ghcr.io}" = "$GHCR_IMAGE" ]; then
  trace "GHCR_IMAGE '$GHCR_IMAGE' has no ghcr.io/ prefix; probing '$repository' as-is"
fi

scope="repository:${repository}:pull"
body="$(mktemp)"
trap 'rm -f "$body"' EXIT

attempt=1
while :; do
  status="$(curl -s -o "$body" -w '%{http_code}' \
    "${GHCR_TOKEN_ENDPOINT}?service=ghcr.io&scope=${scope}" || echo "000")"
  trace "attempt ${attempt}/${retries} -> HTTP ${status}"

  case "$status" in
    200)
      echo "OK: ${GHCR_IMAGE} is public (the GHCR token endpoint issued an anonymous pull token)"
      exit 0
      ;;
    401)
      echo "::error::${GHCR_IMAGE} is PRIVATE -- an anonymous 'docker pull' fails with 'unauthorized'."
      echo "Make the package public: the package page -> Package settings -> Danger Zone -> Change visibility -> Public."
      echo "Package settings are not reachable through the REST API today (github/community discussion #33310), so this is a one-off manual step."
      exit 1
      ;;
    403)
      echo "::error::${GHCR_IMAGE} is not visible to anonymous callers (DENIED) -- the package does not exist, so the push likely never happened."
      exit 1
      ;;
    000 | 5??)
      if [ "$attempt" -ge "$retries" ]; then
        echo "::error::The GHCR token endpoint kept failing for ${GHCR_IMAGE} (last status ${status}) after ${retries} attempt(s)"
        exit 1
      fi
      trace "transient status ${status}; retrying in ${delay}s"
      attempt=$((attempt + 1))
      sleep "$delay"
      ;;
    *)
      echo "::error::Unexpected HTTP ${status} from the GHCR token endpoint for ${GHCR_IMAGE}"
      exit 1
      ;;
  esac
done
