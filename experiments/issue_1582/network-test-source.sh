#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/compact-fixture.py
rustfmt --edition 2024 rust/src/cli_env.rs rust/src/solver_config.rs rust/tests/integration/with_formal_ai.rs
rg -n 'LIVE_FETCH|live_fetch_enabled|with_offline|try_live_web_search' rust/tests/unit/*web* rust/tests/unit/*search* rust/tests/integration/*web* rust/src/solver_handlers/web_requests/live_search.rs | head -100
rg -n 'FORMAL_AI_LIVE_FETCH|FORMAL_AI_OFFLINE' scripts/cargo-test.sh rust/tests/unit.rs rust/tests/integration.rs
git diff --check
