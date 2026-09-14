#!/usr/bin/env bash
# regenerate-derived-artifacts.sh
#
# Rebuild every committed artifact that is derived from other committed sources,
# and restore the canonical order of every union-merged list.
#
# Issue #991: `.gitattributes` marks those paths `merge=union`, so a merge never
# stops on them. The union is a superset of both branches -- it can be stale or
# repeated, never missing -- and this script turns that superset back into the
# one correct file. Run it once after merging; it replaces what used to be a
# hand-resolved conflict in each of these files.
#
# Usage:
#   bash scripts/regenerate-derived-artifacts.sh          # rebuild in place
#   bash scripts/regenerate-derived-artifacts.sh --check  # fail if anything is stale
#
# The registry of what is derived and why is data/meta/merge-conflict-policy.lino.

set -euo pipefail

CHECK_ONLY=0
if [ "${1:-}" = "--check" ]; then
  CHECK_ONLY=1
fi

cd "$(dirname "$0")/.."

run_step() {
  local label="$1"
  shift
  echo "--- $label"
  "$@"
}

run_step "ordered lists (src/lib.rs, tests/*/mod.rs, worker module list)" \
  rust-script scripts/normalize-ordered-lists.rs --write
run_step "seed inventory (src/seed/embedded_registry.rs, src/web/seed-files.js)" \
  rust-script scripts/generate-seed-registry.rs --write
run_step "requirements document (REQUIREMENTS.md)" \
  rust-script scripts/assemble-requirements.rs --write
run_step "total closure (data/seed/closure-generated-*.lino)" \
  python3 scripts/close-total.py
run_step "seed metadata gaps (data/meta/seed-metadata-gaps-*.lino)" \
  rust-script scripts/audit-seed-metadata.rs --write
run_step "hardcoded-language allowlist (scripts/hardcoded-language-allowlist.txt)" \
  rust-script scripts/check-hardcoded-language.rs --write
run_step "self-AST census (data/meta/self-ast/)" \
  cargo run --quiet --example regenerate_self_ast_census
# The reviewed proposal document is derived from the live learner: its candidate
# id and seed edit are content-addressed over the recursive-core trace, so any
# new pipeline stage makes it stale (issue #1073 added the thirteenth). The
# regenerator refreshes only the machine-derived fields and keeps the review
# decisions. `data/seed/learned-methods.lino` is deliberately NOT here: adopting
# a method requires a human-confirmed promotion run, which is the trust boundary
# issue #922 exists to hold.
run_step "reviewed method proposals (examples/issue-922-method-learning/open-proposals.lino)" \
  cargo run --quiet --example regenerate_issue_922_open_proposals
# These three are derived from src/agentic_coding/planner.rs, so any edit to the
# planner -- including one a formatter makes -- leaves them stale. They were
# missing from this list, and the only thing that noticed was a test failure
# several minutes into the suite.
run_step "planner fixtures (data/meta/self-ast.lino, data/meta/self-healing-case.lino, docs/case-studies/issue-538/agent-cli-session-self-ast.json)" \
  cargo run --quiet --example regenerate_planner_fixtures
run_step "formatting" cargo fmt

echo ""
if [ "$CHECK_ONLY" -eq 1 ]; then
  if [ -n "$(git status --porcelain)" ]; then
    echo "::error::Derived artifacts are stale after the merge."
    git status --short
    echo ""
    echo "Run: bash scripts/regenerate-derived-artifacts.sh && git commit -a"
    exit 1
  fi
  echo "Every derived artifact is up to date."
else
  echo "Derived artifacts regenerated. Review 'git status' and commit the result."
fi
