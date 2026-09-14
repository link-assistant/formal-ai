#!/usr/bin/env bash
# Push HEAD to a branch other workflows also write to, and survive losing the race.
#
# Issue #1081 (defect D13). Every job in this repository that writes to `main`
# already shares one concurrency group -- `formal-ai-repository-writes`, with
# `queue: max` -- so two writers never run at the same time. That orders them;
# it does not rebase them.
#
# `actions/checkout` checks out `github.sha`, the commit that triggered the run.
# A writer that waits in the queue is therefore one or more commits behind the
# branch the moment the writer ahead of it lands, and its push is rejected:
#
#     ! [rejected]        main -> main (non-fast-forward)
#
# The serialisation converted "two writers collide" into "the second writer
# reliably fails", which is better because it is deterministic and loud, but it
# is still a lost write. `scripts/version-and-commit.rs` has pulled and retried
# since it was written; the scheduled benchmark ledger push in
# `.github/workflows/external-benchmarks.yml` did not, and it is the writer most
# likely to lose, because it runs on a cron that has no relationship to when a
# release lands. This is the shared implementation for both kinds of caller.
#
# The classification matters as much as the retry. A repository ruleset also
# reports `[rejected]` (GH006, GH013, "Changes must be made through a pull
# request"), and no number of rebases can ever satisfy a rule -- retrying it
# burns the queue slot and then reports the wrong cause. So a rule rejection
# exits immediately, with a message that names the rule rather than the race.
#
# Usage: push-to-shared-branch.sh <remote> <branch>
#
# Exit codes:
#   0   pushed (possibly after a rebase)
#   1   a rejection a rebase cannot fix, or the retries ran out
#   2   usage error
#   3   the rebase itself conflicted; the branch needs a human
#
# Environment:
#   PUSH_MAX_ATTEMPTS          how many pushes to try in total (5)
#   PUSH_RETRY_DELAY_SECONDS   pause before re-reading the remote (3)
#   FORMAL_AI_CI_VERBOSE       `true` echoes each attempt and git's own output
set -euo pipefail

remote="${1:?a remote is required}"
branch="${2:?a branch is required}"

max_attempts="${PUSH_MAX_ATTEMPTS:-5}"
retry_delay="${PUSH_RETRY_DELAY_SECONDS:-3}"
verbose="${FORMAL_AI_CI_VERBOSE:-false}"

log() {
  [ "${verbose}" = "true" ] || return 0
  echo "push-to-shared-branch: $*" >&2
}

# A rule can never be satisfied by a rebase. GitHub reports these as GH006 or
# GH013 with a `- Changes must be made through a pull request` bullet; the
# wording has changed before, so all three spellings are matched.
blocked_by_repository_rule() {
  grep -qE 'GH006|GH013|Changes must be made through a pull request|protected branch' <<<"$1"
}

# The race this script exists for. `fetch first` and `behind its remote
# counterpart` are the hint lines git prints beside the rejection.
rejected_as_non_fast_forward() {
  grep -qE '\[rejected\].*(non-fast-forward|fetch first)|behind its remote counterpart' <<<"$1"
}

attempt=1
while [ "${attempt}" -le "${max_attempts}" ]; do
  log "attempt ${attempt}/${max_attempts}: git push ${remote} HEAD:${branch}"
  if output="$(git push "${remote}" "HEAD:${branch}" 2>&1)"; then
    [ -z "${output}" ] || echo "${output}"
    if [ "${attempt}" -gt 1 ]; then
      echo "::notice title=Shared-branch push retried::Landed on ${branch} at attempt ${attempt} of ${max_attempts} after rebasing onto a concurrent writer."
    fi
    exit 0
  fi
  echo "${output}"

  if blocked_by_repository_rule "${output}"; then
    echo "::error title=Push blocked by a repository rule::${branch} rejects direct pushes, so no number of rebases can land this commit. It has to go through a pull request (see CI-CD-BEST-PRACTICES principle 9)."
    exit 1
  fi

  if ! rejected_as_non_fast_forward "${output}"; then
    # Auth, network, a missing remote: rebasing would hide the real error
    # behind a second, unrelated failure.
    echo "::error title=Push to ${branch} failed::The rejection is not a lost race, so it was not retried. The git output above is the cause."
    exit 1
  fi

  if [ "${attempt}" -eq "${max_attempts}" ]; then
    echo "::error title=Push to ${branch} kept losing the race::${max_attempts} attempts all lost to a concurrent writer. Either the writer queue is saturated or something outside this repository is pushing to ${branch}."
    exit 1
  fi

  log "lost the race; pulling --rebase from ${remote}/${branch}"
  sleep "${retry_delay}"
  if ! rebase_output="$(git pull --rebase "${remote}" "${branch}" 2>&1)"; then
    echo "${rebase_output}"
    # Leave no half-applied rebase behind for a later step to trip over.
    git rebase --abort >/dev/null 2>&1 || true
    echo "::error title=Rebase onto ${branch} conflicted::A concurrent writer changed the same lines this job is committing, so the two edits cannot be ordered automatically."
    exit 3
  fi
  if [ "${verbose}" = "true" ]; then
    echo "${rebase_output}" >&2
  fi
  attempt=$((attempt + 1))
done

# Unreachable: the loop exits inside its own body.
exit 1
