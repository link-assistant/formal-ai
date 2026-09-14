#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI authorship proof for one issue #932 leaf.
#
# The leaf is the box image survey: which link-foundation/box repositories are
# actually published, which are not, and the tag the language matrix pins. It is
# the first smallest leaf of issue #932 ("survey the published images, then pin
# one tag"), and `tests/unit/issue_932_box_language_projects.rs` asserts the
# committed artifact is byte-for-byte what this recipe produces.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-932/self-hosting-authorship"
CANONICAL="$ROOT/data/meta/box-image-survey.lino"
TASK='Finish Formal AI issue #932 by exercising each generated language project inside the matching link-foundation box image. As one smallest leaf of that same task, create file box-image-survey.lino containing exactly
box_image_survey
  record_type "box_image_survey"
  issue "932"
  registry "docker.io"
  namespace "konard"
  surveyed_at "2026-08-14T12:16:42Z"
  pinned_tag "2.4.0"
  evidence "docs/case-studies/issue-932/raw-data/box-image-tags.log"
box_image_published
  record_type "box_image_availability"
  published "true"
  repositories ("box" "box-rust" "box-python" "box-js" "box-go" "box-java" "box-ruby")
box_image_missing
  record_type "box_image_availability"
  published "false"
  repositories ("box-c" "box-cpp" "box-csharp" "box-dotnet")'

TASK="$TASK" \
EXPECT_FILE="box-image-survey.lino" \
EXPECT_TEXT='pinned_tag "2.4.0"' \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8932}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/box-image-survey.lino" "$CANONICAL"
cmp "$ARTIFACT_DIR/box-image-survey.lino" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
