#!/usr/bin/env bash
# Real two-turn Agent CLI proof for issue #936: establish a program plan, resume
# that session, export the verified rewrite to JavaScript/WASM, and execute it.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

TASK="Write me a Rust program that lists the files in the current directory" \
FOLLOW_UP="Sort the results in reverse order and export the substitution rule to JavaScript" \
EXPECT_FILE="program_plan_rules.mjs" \
EXPECT_FILES=$'program_plan_rules_wasm.rs\nprogram_plan_rules.substitution-ir.json\ninput.tsv' \
EXPECT_TEXT="WebAssembly.instantiate" \
EXPECT_SERVER_TEXTS=$'rustup target add wasm32-unknown-unknown\nrustc --edition=2024 --target wasm32-unknown-unknown\nnode program_plan_rules.mjs program_plan_rules.wasm < input.tsv' \
EXPECT_AGENT_TEXTS='request:task\tlist_files_reverse_sort' \
MIN_POSTS=9 \
ATTEMPTS=3 \
PORT="${PORT:-8936}" \
ARTIFACT_DIR="${ARTIFACT_DIR:-}" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"
