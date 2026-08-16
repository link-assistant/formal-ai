#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI proof for one reviewed issue #936 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-936/self-hosting-authorship"
CANONICAL="$ROOT/data/meta/substitution-compiler-contract.lino"
TASK='Implement Formal AI issue #936 by compiling substitution rules through a target-neutral IR to Rust, WebAssembly, and JavaScript interop. As one smallest leaf of that task, create a file substitution-compiler-contract.lino containing exactly:
substitution_compiler_contract
  source substitution_rule_set
  lower target_neutral_ir
  canonical_runtime rust
  webassembly rust_to_wasm
  javascript interop_only
  proof_gate verified_finite_program_plan
  parity interpreter_exact_output'

TASK="$TASK" \
EXPECT_FILE="substitution-compiler-contract.lino" \
EXPECT_TEXT="parity interpreter_exact_output" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8937}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/substitution-compiler-contract.lino" "$CANONICAL"
cmp "$ARTIFACT_DIR/substitution-compiler-contract.lino" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
