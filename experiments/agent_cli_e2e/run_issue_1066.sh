#!/usr/bin/env bash
# Real Agent-CLI proof that a recorded finding says something (issue #1066).
#
# The #1028 agent ladder asks every node for two things at once: do the work,
# and leave the result at a named path whose first line is pinned. That shape
# passed 63/63 while the proof files carried nothing but the pinned line, so the
# mechanical criterion the ladder checks — file exists, is non-empty, first line
# matches — was satisfied by a file that proved nothing.
#
# This leg drives the real `@link-assistant/agent` CLI against `formal-ai serve`
# over the OpenAI-compatible endpoint (no mocks, no in-process shortcut) and
# then reads the file the CLI actually wrote: the body below the pinned line has
# to be an answer about the request's own subject.
#
# Two phrasings run, because a fix that only reads the ladder's wording is not a
# fix (CONTRIBUTING rule 4). Leg 1 is the ladder's own node sentence. Leg 2 is
# an unrelated piece of work, split by a phrasal verb whose object sits between
# its halves — the form English actually uses, and the one the lexicon could not
# read before this issue.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
STAGING="$(mktemp -d)"
trap 'rm -rf "$STAGING"' EXIT

# A proof file is hollow when everything below the pinned line is a heading, a
# bullet marker or whitespace. That is exactly what the green ladder run wrote,
# so it is what this asserts against — by content, not by expected wording.
assert_answers() {
  local file="$1" first_line="$2" subject="$3"
  local body

  if [ "$(head -n 1 "$file")" != "$first_line" ]; then
    echo "issue #1066: $file does not open with the pinned line $first_line" >&2
    head -n 5 "$file" >&2
    return 1
  fi

  body="$(tail -n +2 "$file" | sed 's/^[[:space:]]*[-*#0-9.)]*[[:space:]]*//')"
  if [ "$(printf '%s' "$body" | tr -cd '[:alnum:]' | wc -c)" -lt 40 ]; then
    echo "issue #1066: $file is a heading with no answer under it" >&2
    cat "$file" >&2
    return 1
  fi

  if ! grep -Fqi "$subject" <<<"$body"; then
    echo "issue #1066: $file never mentions $subject, so it answers something else" >&2
    cat "$file" >&2
    return 1
  fi
}

# Leg 1 — the ladder's own node sentence, verbatim in shape.
NODE_TASK='Complete recursive decomposition node 1.2.1, covering atomic tasks L05-L08; both child nodes must produce independently checkable evidence. Leave observable evidence in issue-1066-node-proof.md. The first line must be exactly node_path=1.2.1'

TASK="$NODE_TASK" \
EXPECT_FILE="issue-1066-node-proof.md" \
EXPECT_TEXT="node_path=1.2.1" \
MIN_POSTS=2 \
ATTEMPTS=3 \
PORT="${NODE_PORT:-8944}" \
ARTIFACT_DIR="$STAGING/node" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

assert_answers "$STAGING/node/issue-1066-node-proof.md" 'node_path=1.2.1' 'decomposition'

# Leg 2 — different words, different subject, and a separated phrasal verb.
SPLIT_TASK='Break the customer import rewrite into sub-tasks and record what you work out in import-split.md. The first line must be exactly plan_for=customer-import'

TASK="$SPLIT_TASK" \
EXPECT_FILE="import-split.md" \
EXPECT_TEXT="plan_for=customer-import" \
MIN_POSTS=2 \
ATTEMPTS=3 \
PORT="${SPLIT_PORT:-8945}" \
ARTIFACT_DIR="$STAGING/split" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

assert_answers "$STAGING/split/import-split.md" 'plan_for=customer-import' 'import'

grep -q 'formal-ai/formal-ai' "$STAGING/node/agent-cli.log"
grep -q 'formal-ai/formal-ai' "$STAGING/split/agent-cli.log"

echo "E2E OK: both recorded findings carry an answer, not just their pinned line"
