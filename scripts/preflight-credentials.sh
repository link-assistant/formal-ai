#!/usr/bin/env bash
# Prove the release can publish before the pipeline spends an hour building it.
#
# Issue #1081, principle 16 of docs/CI-CD-BEST-PRACTICES.md ("Prove You Can
# Publish Before You Build"): every credential this repository releases with was
# only ever exercised by the step that used it, at the end of a 90-minute
# `auto-release` job. A revoked crates.io token or an expired Docker Hub access
# token therefore cost a full build before anyone learned the release could not
# happen -- and on a pull request the same absence means nothing at all, because
# forks have no secrets. This script separates those two readings: `--mode
# release` fails the run, `--mode report` states what a release would find.
#
# Every probe follows the same three rules from that principle:
#
#   Probe with a write, not with a login. A registry token endpoint answers 200
#   for scopes it will not honour -- ghcr.io issues a token for any scope, and
#   docker.io answers an anonymous push-scope request with a pull-only `access`
#   claim -- so the push then fails 403. Opening a blob upload session
#   (`POST /v2/<repo>/blobs/uploads/`) and cancelling it (`DELETE <Location>`)
#   is one round trip, stores nothing, and is the only form of the check that
#   is not a guess.
#
#   Report every failure, not the first. One report naming all three missing
#   credentials is worth three runs each naming one, so no probe aborts the
#   script; each records its own verdict and the exit status is decided once.
#
#   Report `unknown`, never a guess. A registry that times out or answers 429
#   has not said the credential is broken. "0 verified, 3 unknown" is
#   actionable; "no failures" is not.
#
# INPUT (environment)
#   PREFLIGHT_MODE            "release" | "report" (--mode wins). Default report.
#   CARGO_REGISTRY_TOKEN
#   CARGO_TOKEN               crates.io token; the first one set is used.
#   CRATE_NAME                crate whose public record is looked up. Default: formal-ai.
#   GHCR_IMAGE                e.g. ghcr.io/owner/name. Empty disables the probe.
#   GITHUB_ACTOR/GITHUB_TOKEN credential for the GHCR probe.
#   DOCKERHUB_IMAGE           e.g. owner/name. Empty disables the probe, exactly
#                             as scripts/configure-dockerhub-publishing.sh does.
#   DOCKERHUB_USERNAME
#   DOCKERHUB_TOKEN
#   PREFLIGHT_RETRIES         attempts on a transport error or 5xx. Default 3.
#   PREFLIGHT_RETRY_DELAY     seconds between attempts. Default 5.
#   PREFLIGHT_VERBOSE         set to 1 to trace every probe on stderr. Off by
#                             default, so a green run stays quiet and a red one
#                             can be re-run with the tracing on.
#   CRATES_IO_API, GHCR_REGISTRY, GHCR_TOKEN_ENDPOINT, DOCKERHUB_REGISTRY,
#   DOCKERHUB_TOKEN_ENDPOINT  endpoint overrides; tests point them at a stub.
#
# Exit status: 0 unless `--mode release` found a failure, or found nothing it
# could verify at all.
set -uo pipefail

mode="${PREFLIGHT_MODE:-report}"
while [ $# -gt 0 ]; do
  case "$1" in
    --mode)
      mode="${2:-}"
      shift 2
      ;;
    --mode=*)
      mode="${1#--mode=}"
      shift
      ;;
    *)
      echo "::error::unknown argument: $1"
      exit 2
      ;;
  esac
done

case "$mode" in
  release | report) ;;
  *)
    echo "::error::--mode must be 'release' or 'report', got '${mode}'"
    exit 2
    ;;
esac

CRATE_NAME="${CRATE_NAME:-formal-ai}"
CRATES_IO_API="${CRATES_IO_API:-https://crates.io/api/v1}"
GHCR_REGISTRY="${GHCR_REGISTRY:-https://ghcr.io}"
GHCR_TOKEN_ENDPOINT="${GHCR_TOKEN_ENDPOINT:-https://ghcr.io/token}"
DOCKERHUB_REGISTRY="${DOCKERHUB_REGISTRY:-https://registry-1.docker.io}"
DOCKERHUB_TOKEN_ENDPOINT="${DOCKERHUB_TOKEN_ENDPOINT:-https://auth.docker.io/token}"
retries="${PREFLIGHT_RETRIES:-3}"
delay="${PREFLIGHT_RETRY_DELAY:-5}"
# crates.io rejects a request without a User-Agent that identifies the caller.
user_agent="formal-ai-release-preflight (https://github.com/${GITHUB_REPOSITORY:-link-assistant/formal-ai})"

verified=()
failed=()
unknown=()

trace() {
  if [ "${PREFLIGHT_VERBOSE:-}" = "1" ]; then
    echo "preflight: $*" >&2
  fi
}

record() { # record <verified|failed|unknown> <check> <detail>
  case "$1" in
    verified) verified+=("$2 -- $3") ;;
    failed) failed+=("$2 -- $3") ;;
    unknown) unknown+=("$2 -- $3") ;;
  esac
  trace "$1: $2 -- $3"
}

# Headers for the next request, set by the caller immediately before it calls
# `http_status`. They travel to curl through a config document on stdin rather
# than through `-H` arguments, because an argument list is not private: every
# process on the runner can read /proc/<pid>/cmdline while curl is alive, and
# `ps` prints it. A publish token in argv is a secret published to the machine.
curl_headers=()

# Quote one value the way `curl -K` reads it: a double-quoted string in which
# backslash escapes the next character.
curl_quote() { # curl_quote <value>
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  printf '%s' "$value"
}

curl_config() { # curl_config -- the document curl reads on stdin via `-K -`
  local header
  for header in ${curl_headers+"${curl_headers[@]}"}; do
    printf 'header = "%s"\n' "$(curl_quote "$header")"
  done
}

# Echo the HTTP status of one request, or 000 when curl could not speak at all.
# Retries only what a retry can fix: a transport error, a 5xx or a 429.
# Consumes ${curl_headers[@]}; the caller sets it before every call.
http_status() { # http_status <body-file> [curl args...]
  local body="$1"
  shift
  local attempt=1 status config
  config="$(curl_config)"
  while :; do
    status="$(printf '%s' "$config" | curl -sS -o "$body" -w '%{http_code}' -A "$user_agent" -K - "$@" 2>/dev/null)" || status=000
    case "$status" in
      000 | 429 | 5??)
        if [ "$attempt" -ge "$retries" ]; then break; fi
        trace "transient HTTP ${status}; retry ${attempt}/${retries} in ${delay}s"
        attempt=$((attempt + 1))
        sleep "$delay"
        ;;
      *) break ;;
    esac
  done
  printf '%s' "$status"
}

json_field() { # json_field <file> <key>
  grep -o "\"$2\":\"[^\"]*\"" "$1" 2>/dev/null | head -n 1 | cut -d'"' -f4
}

check_crates_io() {
  local check="crates.io publish token"
  local token="${CARGO_REGISTRY_TOKEN:-${CARGO_TOKEN:-}}"
  if [ -z "$token" ]; then
    record failed "$check" "neither CARGO_REGISTRY_TOKEN nor CARGO_TOKEN is set; \`cargo publish\` cannot run"
    return
  fi

  # crates.io has no read endpoint that accepts an API token. `GET /api/v1/me`
  # is declared `AuthCheck::only_cookie()` in crates.io's `src/controllers/
  # user/me.rs`, so every API token -- valid, revoked or expired -- is answered
  # 403 there; the probe that shipped with issue #1081 read that 403 as "the
  # token is revoked, expired or misscoped" and blocked the release of run
  # 34149311523 with a token that had published v0.347.0 two days earlier
  # (issue #1085). Endpoint-scoped tokens are rejected on every route that
  # does not require their scope, and the only route that requires the
  # publish scope is the publish itself. So the token is never sent anywhere
  # from here: the one honest read-only verdict is `unknown`, stated with the
  # reason, and `cargo publish` remains the step that proves it.
  local body status
  body="$(mktemp)"
  curl_headers=()
  status="$(http_status "$body" "${CRATES_IO_API}/crates/${CRATE_NAME}")"
  trace "crates.io crate lookup -> HTTP ${status}"
  case "$status" in
    200 | 404)
      record unknown "$check" "token present; crates.io offers no token-accepting read endpoint (GET /api/v1/me is cookie-only), so only \`cargo publish\` can prove it"
      ;;
    *)
      record unknown "$check" "token present; crates.io answered HTTP ${status} to a public crate lookup, so neither the registry nor the token could be checked"
      ;;
  esac
  rm -f "$body"
}

# Open and immediately cancel a blob upload session: the smallest request a
# registry answers differently for "may push" and "may pull".
probe_registry_push() { # probe_registry_push <check> <registry> <token-endpoint> <service> <repository> <user> <password>
  local check="$1" registry="$2" token_endpoint="$3" service="$4" repository="$5" user="$6" password="$7"
  local body headers status basic bearer location
  body="$(mktemp)"
  headers="$(mktemp)"
  basic="$(printf '%s:%s' "$user" "$password" | base64 | tr -d '\n')"

  curl_headers=("Authorization: Basic ${basic}")
  status="$(http_status "$body" \
    "${token_endpoint}?service=${service}&scope=repository:${repository}:pull,push")"
  curl_headers=()
  trace "${check}: token endpoint -> HTTP ${status}"
  if [ "$status" != "200" ]; then
    if [ "$status" = "401" ] || [ "$status" = "403" ]; then
      record failed "$check" "${service} rejected the credential at its token endpoint (HTTP ${status})"
    else
      record unknown "$check" "${service} answered HTTP ${status} at its token endpoint"
    fi
    rm -f "$body" "$headers"
    return
  fi

  bearer="$(json_field "$body" token)"
  if [ -z "$bearer" ]; then bearer="$(json_field "$body" access_token)"; fi
  if [ -z "$bearer" ]; then
    record unknown "$check" "${service} issued a 200 with no token in it"
    rm -f "$body" "$headers"
    return
  fi

  curl_headers=("Authorization: Bearer ${bearer}" "Content-Length: 0")
  status="$(http_status /dev/null -D "$headers" -X POST \
    "${registry}/v2/${repository}/blobs/uploads/")"
  curl_headers=()
  trace "${check}: blob upload session -> HTTP ${status}"
  case "$status" in
    201 | 202)
      record verified "$check" "${registry} opened a blob upload session for ${repository}"
      location="$(grep -i '^location:' "$headers" | tail -n 1 | cut -d' ' -f2- | tr -d '\r')"
      if [ -n "$location" ]; then
        case "$location" in
          /*) location="${registry}${location}" ;;
        esac
        trace "${check}: cancelling the session"
        curl_headers=("Authorization: Bearer ${bearer}")
        http_status /dev/null -X DELETE "$location" > /dev/null
        curl_headers=()
      fi
      ;;
    401 | 403)
      record failed "$check" "${registry} refused a push to ${repository} (HTTP ${status}); the token authenticates but cannot write"
      ;;
    404)
      record unknown "$check" "${registry} answered 404 for ${repository}; the repository may not exist yet"
      ;;
    *)
      record unknown "$check" "${registry} answered HTTP ${status} when opening a blob upload session"
      ;;
  esac
  rm -f "$body" "$headers"
}

check_ghcr() {
  local check="GHCR push"
  local image="${GHCR_IMAGE:-}"
  if [ -z "$image" ]; then
    record failed "$check" "GHCR_IMAGE is not set, so the release has no image to push"
    return
  fi
  local repository="${image#ghcr.io/}"
  repository="${repository%%@*}"
  repository="${repository%%:*}"
  if [ -z "${GITHUB_TOKEN:-}" ]; then
    record failed "$check" "GITHUB_TOKEN is not set; the job token is what pushes to ghcr.io"
    return
  fi
  probe_registry_push "$check" "$GHCR_REGISTRY" "$GHCR_TOKEN_ENDPOINT" "ghcr.io" \
    "$repository" "${GITHUB_ACTOR:-github-actions}" "$GITHUB_TOKEN"
}

check_dockerhub() {
  local check="Docker Hub push"
  local image="${DOCKERHUB_IMAGE:-}"
  if [ -z "${DOCKERHUB_TOKEN:-}" ]; then
    # Docker Hub publishing is opt-in and the token opts in
    # (scripts/configure-dockerhub-publishing.sh skips it quietly), so an unset
    # token is a configuration choice, not a missing credential. Issue #1131:
    # this keyed off the image until the image got a default, which would have
    # made every fork fail this check instead of skipping it.
    trace "DOCKERHUB_TOKEN is unset; Docker Hub publishing is disabled"
    return
  fi
  if [ -z "$image" ] || [ -z "${DOCKERHUB_USERNAME:-}" ]; then
    record failed "$check" "DOCKERHUB_TOKEN is set but DOCKERHUB_IMAGE/DOCKERHUB_USERNAME are not; the release would fail half-published"
    return
  fi
  probe_registry_push "$check" "$DOCKERHUB_REGISTRY" "$DOCKERHUB_TOKEN_ENDPOINT" "registry.docker.io" \
    "${image%%:*}" "$DOCKERHUB_USERNAME" "$DOCKERHUB_TOKEN"
}

check_crates_io
check_ghcr
check_dockerhub

emit() { # emit <marker> <heading> <array items...>
  local marker="$1" heading="$2"
  shift 2
  local item
  for item in "$@"; do
    if [ "$marker" = "none" ]; then
      echo "  ${heading}: ${item}"
    else
      echo "::${marker}::${heading}: ${item}"
    fi
  done
}

echo "Release preflight (${mode} mode): ${#verified[@]} verified, ${#failed[@]} failed, ${#unknown[@]} unknown"
emit none "OK" ${verified+"${verified[@]}"}
if [ "$mode" = "release" ]; then
  emit error "BLOCKED" ${failed+"${failed[@]}"}
else
  emit warning "would block a release" ${failed+"${failed[@]}"}
fi
emit warning "UNKNOWN" ${unknown+"${unknown[@]}"}

if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  {
    echo "### Release preflight (${mode} mode)"
    echo
    echo "${#verified[@]} verified, ${#failed[@]} failed, ${#unknown[@]} unknown"
    echo
    summarize() { # summarize <label> <items...>
      local label="$1"
      shift
      local item
      for item in "$@"; do
        echo "- ${label} ${item}"
      done
    }
    summarize OK ${verified+"${verified[@]}"}
    summarize FAILED ${failed+"${failed[@]}"}
    summarize UNKNOWN ${unknown+"${unknown[@]}"}
  } >> "$GITHUB_STEP_SUMMARY"
fi

if [ "$mode" != "release" ]; then
  echo "Report mode: a pull request cannot see a fork's secrets, so nothing here fails the run."
  exit 0
fi

if [ "${#failed[@]}" -gt 0 ]; then
  echo "::error::${#failed[@]} release credential(s) cannot publish; not spending a build on a release that cannot happen"
  exit 1
fi

if [ "${#verified[@]}" -eq 0 ]; then
  echo "::error::nothing could be verified (${#unknown[@]} unknown); a run that verified no credential is not a pass"
  exit 1
fi

if [ "${#unknown[@]}" -gt 0 ]; then
  echo "::warning::${#verified[@]} verified, ${#unknown[@]} unknown -- proceeding, but the unknown probes above did not confirm anything"
fi
