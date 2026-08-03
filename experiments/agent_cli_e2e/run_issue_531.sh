#!/usr/bin/env bash
# Real Agent CLI -> formal-ai server -> public learning CLI replay for issue #531.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OBSERVATIONS="$ROOT/data/benchmarks/issue-531-algorithm-traces.lino"
TASK="Discover a reusable algorithm from these execution observations, preserve the held-out evidence, read the saved artifact back, and conformance-check the same task without executing it.

$(<"$OBSERVATIONS")"

PATH="$ROOT/target/release:$PATH" \
TASK="$TASK" \
EXPECT_FILE="discovered-algorithms.lino" \
EXPECT_TEXT='status "held_out_validated"' \
EXPECT_SERVER_TEXTS=$'formal-ai learn algorithms\nformal-ai algorithm conformance' \
MIN_POSTS=5 \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"
